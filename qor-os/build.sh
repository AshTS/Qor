#! /usr/bin/bash
if test ! -d ../userland/bin
then
  mkdir ../userland/bin
fi

echo "Building Userland Programs..."

cd ../libc

make $1 -q
if test $? -ne 0
then
  echo "Building LibC"
  make $1

  cp bin/libc.a ../userland/bin/libc.a
fi

cp include/* ../userland/include/libc/ -r

cd ../userland

for i in slib libcold libgraphics libelf term shell prog hello libc-test pwd basic cat ls clear mkdir checkers bmp ps kill fractal readelf
do
    cd $i
    make $1 -q
    if test $? -ne 0
    then
      echo "Building " $i
      make $1
    fi
    cd ..
done

cd ../qor-os

sudo losetup /dev/loop11 hdd.dsk

sudo mount /dev/loop11 /mnt
sudo rm -rf /mnt/*
sudo cp -r ../userland/bin/ /mnt/bin/
sudo cp -r ../userland/root/ /mnt/

sudo sync /mnt

sudo umount /mnt

sudo losetup -d /dev/loop11

echo "Done."