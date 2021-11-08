CC = clang
CFLAGS = --target=riscv64 -march=rv64gc -mno-relax -fPIC -Wno-unused-command-line-argument
INCLUDE = -isystem ../include

LINK = ld.lld
LINKFLAGS = -r

INCLUDES = ../include/printf.h

LIB_DIR = ../bin
OUTPUT_DIR = ../bin
BUILD_DIR = build
SRC_DIR = src

_LIBS = 
LIBS = $(patsubst %,$(LIB_DIR)/%,$(_LIBS))

_OBJ = syscalls.o malloc.o printf.o string.o
OBJ = $(patsubst %,$(BUILD_DIR)/%,$(_OBJ))

$(OUTPUT_DIR)/libcold : $(BUILD_DIR) $(OBJ) $(LIBS)
	$(LINK) $(LINKFLAGS) $(OBJ) $(LIBS) -o $@

$(BUILD_DIR)/%.o : $(SRC_DIR)/%.c $(INCLUDES)
	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@

$(BUILD_DIR)/%.o : $(SRC_DIR)/%.s $(INCLUDES)
	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@

$(BUILD_DIR) :
	[ ! -d "$(BUILD_DIR)" ] && mkdir $(BUILD_DIR)


.PHONY: clean

clean:
	rm -rf build/*
