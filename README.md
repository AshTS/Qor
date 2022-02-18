# Qor

Basic kernel written in Rust for RISC-V, following the tutorial by Stephen Marz [RISC-V OS using Rust](https://osblog.stephenmarz.com/index.html).

Stored in seperate repositories are the [libc implementation](https://github.com/CarterTS/QorLibC) and the [userland programs](https://github.com/CarterTS/QorUserland)

## Install

1. Clone the repository and all submodules
2. Navigate to the `qemu` directory
3. Create a new directory called `build` and navigate there
4. Run `../configure --target-list=riscv64-softmmu --enable-sdl && make`
5. Install qemu to your path
6. Navigate to the `qor-os` directory
8. Run `rustup override set nightly` to set the Rust compiler version
9. Run `rustup target add riscv64gc-unknown-none-elf` to install the proper target
10. Run `rustup component add rust-src` to allow the core library to be built

### First Time Execution

Before the first execution, the hard disk must be created using the following in the `qor-os` directory:

```
fallocate -l 32M hdd.dsk
sudo losetup /dev/loop11 hdd.dsk
sudo mkfs.minix -3 /dev/loop11
sudo losetup -d /dev/loop11
```

The userland programs must also be built before the first execution, to do so run `./build.py rebuild` in the root directory.

## Usage

To start the kernel, run `./build.py run` in the root directory.


## License from Tutorial

MIT License

Copyright (c) 2019 Stephen Marz

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
