#! /usr/bin/fish

cargo clean
cargo objdump -- -D > dis.txt