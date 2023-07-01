#! /usr/bin/bash

qemu-system-riscv64 -D ./log.txt -d int -machine virt -cpu rv64 -d guest_errors,unimp -smp $CORE_COUNT -m 128M -drive if=none,format=raw,file=hdd.dsk,id=foo -device virtio-blk-device,scsi=off,drive=foo -serial mon:stdio -bios none -device virtio-rng-device -device virtio-gpu-device -device virtio-net-device -device virtio-tablet-device -device virtio-keyboard-device -kernel $1
exit $?