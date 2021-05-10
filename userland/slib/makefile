../bin/slib.o : build/slib.o build/printf.o build
	ld.lld -r build/slib.o build/printf.o -o ../bin/slib.o

build/slib.o : slib.s build
	clang --target=riscv64 -march=rv64gc -mno-relax -c slib.s -o build/slib.o

build/printf.o : printf.c build
	clang --target=riscv64 -march=rv64gc -mno-relax -c printf.c -o build/printf.o

build:
	[ ! -d "build" ] && mkdir build

clean:
	rm -rf build