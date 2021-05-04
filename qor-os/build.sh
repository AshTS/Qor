#! /usr/bin/fish

cd ../userland/prog

cargo build

cd ../../qor-os

sudo mount /dev/loop11 /mnt

sudo cp ../userland/prog/target/riscv64gc-unknown-none-elf/debug/prog /mnt/bin/prog

sudo sync
sudo umount /mnt
