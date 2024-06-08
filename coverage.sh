#! /bin/sh -e
#
# Generate a code coverage report
#
# Requirements:
# sudo pkg install grcov
# cargo install grcov
# rustup component add llvm-tools-preview
#
# Usage:
# coverage.sh

export LLVM_PROFILE_FILE="mdconfig-%p-%m.profraw"
export RUSTFLAGS="-Cinstrument-coverage"
TOOLCHAIN=nightly
cargo +$TOOLCHAIN build --all-targets
sudo -E cargo +$TOOLCHAIN test

grcov . --binary-path $PWD/target/debug -s src -t html --branch \
	--ignore 'ffi*.rs' \
	--ignore-not-existing \
	-o ./coverage/
