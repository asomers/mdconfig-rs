use std::{
    ffi::OsStr,
    fs,
    mem,
    os::{
        fd::AsRawFd,
        unix::{ffi::OsStrExt, fs::FileTypeExt},
    },
    path::Path,
    process::Command,
    sync::OnceLock,
};

use cfg_if::cfg_if;
use mdconfig::*;
use nix::{ioctl_read, ioctl_readwrite};

cfg_if! {
    if #[cfg(target_pointer_width = "64")] {
        mod ffi64;
        use ffi64 as ffi;
    } else if #[cfg(target_pointer_width = "32")] {
        mod ffi32;
        use ffi32 as ffi;
    }
}

static FBSD15: OnceLock<bool> = OnceLock::new();

#[macro_export]
macro_rules! require_fbsd15 {
    () => {
        let fbsd15 = FBSD15.get_or_init(|| {
            let major = nix::sys::utsname::uname()
                .unwrap()
                .release()
                .to_str()
                .unwrap()
                .split('.')
                .next()
                .unwrap()
                .parse::<i32>()
                .unwrap();
            major >= 15
        });
        if !fbsd15 {
            use ::std::io::Write;

            let stderr = ::std::io::stderr();
            let mut handle = stderr.lock();
            writeln!(
                handle,
                "This test requires FreeBSD 15 or later.  Skipping test."
            )
            .unwrap();
            return;
        }
    };
}

ioctl_read!(diocgsectorsize, 'd', 128, nix::libc::c_uint);
ioctl_read!(diocfwsectors, 'd', 130, nix::libc::c_uint);
ioctl_read!(diocfwheads, 'd', 131, nix::libc::c_uint);
ioctl_readwrite!(diocgattr, 'd', 142, ffi::diocgattr_arg);

#[derive(Clone, Debug)]
struct MdData {
    name:    String,
    type_:   String,
    size:    String,
    path:    String,
    label:   String,
    options: String,
}

fn list_unit(unit: u32) -> MdData {
    let output = Command::new("mdconfig")
        .arg("-lvu")
        .arg(format!("{}", unit))
        .output()
        .unwrap();
    let line = OsStr::from_bytes(&output.stdout)
        .to_string_lossy()
        .to_string();
    let mut fields = line.split_ascii_whitespace();
    MdData {
        name:    fields.next().unwrap().to_string(),
        type_:   fields.next().unwrap().to_string(),
        size:    fields.next().unwrap().to_string(),
        path:    fields.next().unwrap().to_string(),
        label:   fields.next().unwrap_or("-").to_string(),
        options: fields.next().unwrap_or("").to_string(),
    }
}

mod create {
    use super::*;

    #[test]
    fn async_() {
        require_fbsd15!();

        let tf = tempfile::NamedTempFile::new().unwrap();
        tf.as_file().set_len(1 << 21).unwrap();
        let md = Builder::vnode(tf.path()).async_(true).create().unwrap();

        let data = list_unit(md.unit());
        assert_eq!(data.options, "async");
    }

    #[test]
    fn cache() {
        require_fbsd15!();

        let tf = tempfile::NamedTempFile::new().unwrap();
        tf.as_file().set_len(1 << 21).unwrap();
        let md = Builder::vnode(tf.path()).cache(true).create().unwrap();

        let data = list_unit(md.unit());
        assert_eq!(data.options, "cache");
    }

    #[test]
    fn compress() {
        require_fbsd15!();

        let md = Builder::malloc(1 << 20).compress(true).create().unwrap();

        let data = list_unit(md.unit());
        assert_eq!(data.options, "compress");
    }

    #[test]
    fn dev() {
        let md = Builder::null(1 << 20).create().unwrap();

        // Check that the device type is as expected
        let metadata = fs::metadata(md.path()).unwrap();
        assert!(metadata.file_type().is_char_device());
    }

    #[test]
    fn label() {
        let md = Builder::null(1 << 20).label("foo").create().unwrap();

        let data = list_unit(md.unit());
        assert_eq!(data.label, "foo");
    }

    #[test]
    fn malloc() {
        let md = Builder::malloc(1 << 20).create().unwrap();

        let data = list_unit(md.unit());
        assert_eq!(data.name, md.name());
        assert_eq!(data.type_, "malloc");
        assert_eq!(data.size, "1024K");
        assert_eq!(data.path, "-");
        assert_eq!(data.label, "-");
    }

    #[test]
    fn mustdealloc() {
        require_fbsd15!();

        let tf = tempfile::NamedTempFile::new().unwrap();
        tf.as_file().set_len(1 << 21).unwrap();
        let md = Builder::vnode(tf.path())
            .mustdealloc(true)
            .create()
            .unwrap();

        let data = list_unit(md.unit());
        assert_eq!(data.options, "mustdealloc");
    }

    #[test]
    fn name() {
        let md = Builder::null(1 << 20).create().unwrap();
        assert_eq!(&format!("md{}", md.unit()), md.name());
    }

    #[test]
    fn null() {
        let md = Builder::null(1 << 20).create().unwrap();

        let data = list_unit(md.unit());
        assert_eq!(data.name, md.name());
        assert_eq!(data.type_, "null");
        assert_eq!(data.size, "1024K");
        assert_eq!(data.path, "-");
        assert_eq!(data.label, "-");
    }

    #[test]
    fn readonly() {
        require_fbsd15!();

        let tf = tempfile::NamedTempFile::new().unwrap();
        tf.as_file().set_len(1 << 21).unwrap();
        let md = Builder::vnode(tf.path()).readonly(true).create().unwrap();

        let data = list_unit(md.unit());
        assert_eq!(data.options, "readonly");
    }

    #[test]
    fn reserve() {
        require_fbsd15!();

        let md = Builder::swap(1 << 20).reserve(true).create().unwrap();

        let data = list_unit(md.unit());
        assert_eq!(data.options, "reserve");
    }

    #[test]
    fn sectorsize() {
        let md = Builder::swap(1 << 20).sectorsize(2048).create().unwrap();
        let mut sectorsize = 0u32;
        let f = fs::File::open(md.path()).unwrap();
        unsafe { diocgsectorsize(f.as_raw_fd(), &mut sectorsize).unwrap() };
        assert_eq!(sectorsize, 2048);
    }

    // The kernel requires both of sectors_per_track and heads to be set.  If only one is set, it
    // ignores it.
    #[test]
    fn sectors_per_track_and_heads() {
        let md = Builder::swap(1 << 30)
            .sectors_per_track(42)
            .heads_per_cylinder(69)
            .create()
            .unwrap();
        let mut sectors = 0u32;
        let mut heads = 0u32;
        let f = fs::File::open(md.path()).unwrap();
        unsafe {
            diocfwsectors(f.as_raw_fd(), &mut sectors).unwrap();
            diocfwheads(f.as_raw_fd(), &mut heads).unwrap();
        }
        drop(f);
        assert_eq!(sectors, 42);
        assert_eq!(heads, 69);
    }

    #[test]
    fn swap() {
        let md = Builder::swap(1 << 20).create().unwrap();

        let data = list_unit(md.unit());
        assert_eq!(data.name, md.name());
        assert_eq!(data.type_, "swap");
        assert_eq!(data.size, "1024K");
        assert_eq!(data.path, "-");
        assert_eq!(data.label, "-");
    }

    #[test]
    fn unit() {
        let md = Builder::null(1 << 20).unit(666).create().unwrap();

        assert_eq!(md.unit(), 666);
        list_unit(md.unit());
    }

    #[test]
    fn verify() {
        let tf = tempfile::NamedTempFile::new().unwrap();
        tf.as_file().set_len(1 << 21).unwrap();
        let md = Builder::vnode(tf.path()).verify(true).create().unwrap();

        let f = fs::File::open(md.path()).unwrap();
        let attrname = OsStr::new("MNT::verified");
        let verified = unsafe {
            let mut arg: ffi::diocgattr_arg = mem::zeroed();
            arg.len = mem::size_of::<libc::c_int>() as i32;
            let attrp = attrname.as_bytes().as_ptr() as *const i8;
            arg.name.as_mut_ptr().copy_from(attrp, attrname.len());
            let r = diocgattr(f.as_raw_fd(), &mut arg);
            cfg_if! {
                if #[cfg(target_pointer_width = "32")] {
                    if r == Err(nix::errno::Errno::ENOTTY) {
                        // This error usually means that we're running in 32-bit emulation mode.
                        // DIOCGATTR does not work in 32-bit emulation, so skip this test.
                        return
                    }
                }
            }
            r.unwrap();
            arg.value.i
        };
        assert!(verified != 0);
        drop(f);
    }

    #[test]
    fn vnode() {
        let tf = tempfile::NamedTempFile::new().unwrap();
        tf.as_file().set_len(1 << 21).unwrap();
        let md = Builder::vnode(tf.path()).create().unwrap();

        let data = list_unit(md.unit());
        assert_eq!(data.name, md.name());
        assert_eq!(data.type_, "vnode");
        assert_eq!(data.size, "2048K");
        assert_eq!(Path::new(&data.path), tf.path());
    }

    /// Create a vnode-backed MD device, but override the default size
    #[test]
    fn vnode_with_size() {
        let tf = tempfile::NamedTempFile::new().unwrap();
        tf.as_file().set_len(1 << 21).unwrap();
        let md = Builder::vnode(tf.path()).size(1 << 20).create().unwrap();

        let data = list_unit(md.unit());
        assert_eq!(data.size, "1024K");
    }
}

mod drop {
    use super::*;

    /// Test that the device gets deallocated on drop
    #[test]
    fn deallocate_on_drop() {
        let md = Builder::null(1 << 20).create().unwrap();
        let path = md.path().to_owned();
        let oldstat = fs::metadata(&path).unwrap();
        let old_mtime = oldstat.modified().unwrap();
        drop(md);

        // Check the path again.  Either it should not exist, or its mtime
        // should've changed.
        if let Ok(newstat) = fs::metadata(&path) {
            let new_mtime = newstat.modified().unwrap();
            assert!(old_mtime != new_mtime);
        }
    }
}

mod resize {
    use super::*;

    #[test]
    fn down() {
        let md = Builder::swap(1 << 21).create().unwrap();

        let data = list_unit(md.unit());
        assert_eq!(data.size, "2048K");

        md.resize(1 << 20, true).unwrap();
        let data = list_unit(md.unit());
        assert_eq!(data.size, "1024K");
    }

    #[test]
    fn up() {
        let md = Builder::swap(1 << 20).create().unwrap();

        let data = list_unit(md.unit());
        assert_eq!(data.size, "1024K");

        md.resize(1 << 21, false).unwrap();
        let data = list_unit(md.unit());
        assert_eq!(data.size, "2048K");
    }
}

mod try_destroy {
    use std::{
        thread::sleep,
        time::{Duration, Instant},
    };

    use super::*;

    #[test]
    fn ebusy() {
        let md = Builder::swap(1 << 21).create().unwrap();
        let _f = fs::File::open(md.path()).unwrap();
        let (_md, e) = md.try_destroy().unwrap_err();
        assert_eq!(libc::EBUSY, e.raw_os_error().unwrap());
    }

    #[test]
    fn ok() {
        let mut md = Builder::swap(1 << 21).create().unwrap();
        let timeout = Duration::from_secs(5);
        let start = Instant::now();
        loop {
            md = match md.try_destroy() {
                Ok(()) => return,
                Err((md, _)) => md,
            };
            if start.elapsed() > timeout {
                panic!("Could not destroy within {:?}", timeout);
            }
            sleep(Duration::from_millis(50));
        }
    }
}
