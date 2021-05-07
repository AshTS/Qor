#! /usr/bin/fish

cd ../userland/prog

cargo build

cd ../../qor-os

sudo losetup /dev/loop11 hdd.dsk

sudo mount /dev/loop11 /mnt
sudo cp ../userland/prog/target/riscv64gc-unknown-none-elf/debug/prog /mnt/bin/prog

stat /mnt/bin/prog

sudo sync /mnt

sudo umount /mnt

sudo losetup -d /dev/loop11