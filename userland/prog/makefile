../bin/prog: build main.c ../include/syscalls.h ../bin/slib.o
	clang --target=riscv64 -march=rv64gc -mno-relax -isystem ../include -c main.c -o build/main.o 
	ld.lld build/main.o ../bin/slib.o -o ../bin/prog

build:
	[ ! -d "build" ] && mkdir build

clean:
	rm -rf build