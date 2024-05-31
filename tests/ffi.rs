#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
/* automatically generated by rust-bindgen 0.69.4 */

pub type __int64_t = ::std::os::raw::c_long;
pub type __off_t = __int64_t;
pub type off_t = __off_t;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct diocgattr_arg {
    pub name:  [::std::os::raw::c_char; 64usize],
    pub len:   ::std::os::raw::c_int,
    pub value: diocgattr_arg__bindgen_ty_1,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union diocgattr_arg__bindgen_ty_1 {
    pub str_: [::std::os::raw::c_char; 256usize],
    pub off:  off_t,
    pub i:    ::std::os::raw::c_int,
    pub u16_: u16,
}
#[test]
fn bindgen_test_layout_diocgattr_arg__bindgen_ty_1() {
    const UNINIT: ::std::mem::MaybeUninit<diocgattr_arg__bindgen_ty_1> =
        ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<diocgattr_arg__bindgen_ty_1>(),
        256usize,
        concat!("Size of: ", stringify!(diocgattr_arg__bindgen_ty_1))
    );
    assert_eq!(
        ::std::mem::align_of::<diocgattr_arg__bindgen_ty_1>(),
        8usize,
        concat!("Alignment of ", stringify!(diocgattr_arg__bindgen_ty_1))
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).str_) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(diocgattr_arg__bindgen_ty_1),
            "::",
            stringify!(str_)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).off) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(diocgattr_arg__bindgen_ty_1),
            "::",
            stringify!(off)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).i) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(diocgattr_arg__bindgen_ty_1),
            "::",
            stringify!(i)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).u16_) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(diocgattr_arg__bindgen_ty_1),
            "::",
            stringify!(u16_)
        )
    );
}
#[test]
fn bindgen_test_layout_diocgattr_arg() {
    const UNINIT: ::std::mem::MaybeUninit<diocgattr_arg> =
        ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<diocgattr_arg>(),
        328usize,
        concat!("Size of: ", stringify!(diocgattr_arg))
    );
    assert_eq!(
        ::std::mem::align_of::<diocgattr_arg>(),
        8usize,
        concat!("Alignment of ", stringify!(diocgattr_arg))
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).name) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(diocgattr_arg),
            "::",
            stringify!(name)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).len) as usize - ptr as usize },
        64usize,
        concat!(
            "Offset of field: ",
            stringify!(diocgattr_arg),
            "::",
            stringify!(len)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).value) as usize - ptr as usize },
        72usize,
        concat!(
            "Offset of field: ",
            stringify!(diocgattr_arg),
            "::",
            stringify!(value)
        )
    );
}
