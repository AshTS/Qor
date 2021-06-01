#! /usr/bin/fish
if not test -d ../userland/bin
  mkdir ../userland/bin
end

set programs slib shell prog hello libc-test pwd

cd ../userland

for i in $programs
    cd $i
    make $argv
    cd ..
end 

cd ../qor-os

sudo losetup /dev/loop11 hdd.dsk

sudo mount /dev/loop11 /mnt
sudo rm -rf /mnt/*
sudo cp -r ../userland/bin/ /mnt/bin/
sudo cp -r ../userland/root/ /mnt/

ls /mnt/bin

sudo sync /mnt

sudo umount /mnt

sudo losetup -d /dev/loop11