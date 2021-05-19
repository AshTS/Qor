#! /usr/bin/fish
# Create a new userland program

set name $argv[1]

mkdir $name

cd $name

mkdir build
mkdir src

touch src/main.c

echo "#include \"printf.h\"" > src/main.c
echo "" >> src/main.c
echo "int main()" >> src/main.c
echo "{" >> src/main.c
echo "    printf(\"Hello World!\n\");" >> src/main.c
echo "    return 0;" >> src/main.c
echo "}" >> src/main.c

touch makefile
echo 'CC = clang' > makefile
echo 'CFLAGS = --target=riscv64 -march=rv64gc -mno-relax' >> makefile
echo 'INCLUDE = -isystem ../include' >> makefile
echo '' >> makefile
echo 'LINK = ld.lld' >> makefile
echo 'LINKFLAGS =' >> makefile
echo '' >> makefile
echo 'INCLUDES = ../include/printf.h' >> makefile
echo '' >> makefile
echo 'LIB_DIR = ../bin' >> makefile
echo 'OUTPUT_DIR = ../bin' >> makefile
echo 'BUILD_DIR = build' >> makefile
echo 'SRC_DIR = src' >> makefile
echo '' >> makefile
echo '_LIBS = slib' >> makefile
echo 'LIBS = $(patsubst %,$(LIB_DIR)/%,$(_LIBS))' >> makefile
echo '' >> makefile
echo '_OBJ = main.o' >> makefile
echo 'OBJ = $(patsubst %,$(BUILD_DIR)/%,$(_OBJ))' >> makefile
echo '' >> makefile
echo '$(OUTPUT_DIR)/'$name' : $(BUILD_DIR) $(OBJ)' >> makefile
echo '	$(LINK) $(LINKFLAGS) $(OBJ) $(LIBS) -o $@' >> makefile
echo '' >> makefile
echo '$(BUILD_DIR)/%.o : $(SRC_DIR)/%.c $(INCLUDES)' >> makefile
echo '	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@' >> makefile
echo '' >> makefile
echo '$(BUILD_DIR)/%.o : $(SRC_DIR)/%.s $(INCLUDES)' >> makefile
echo '	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@' >> makefile
echo '' >> makefile
echo '$(BUILD_DIR) :' >> makefile
echo '	[ ! -d "$(BUILD_DIR)" ] && mkdir $(BUILD_DIR)' >> makefile
echo '' >> makefile
echo '' >> makefile
echo '.PHONY: clean' >> makefile
echo '' >> makefile
echo 'clean:' >> makefile
echo '	rm -rf build/*' >> makefile