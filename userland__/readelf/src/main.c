#include "printf.h"
#include "libelf.h"
#include "syscalls.h"
#include "string.h"
#include "stdbool.h"
#include "malloc.h"

void tabulate(char* s, int length, bool left_justify, char space)
{
    int taken = strlen(s);

    if (left_justify)
    {
        printf("%s", s);
    }

    for (int i = taken; i < length; i++)
    {
        printf("%c", space);
    }

    if (!left_justify)
    {
        printf("%s", s);
    }
}

char* section_type_str(int);

int main(int argc, char** argv)
{
    if (argc < 2)
    {
        eprintf("readelf requires a file\n");
        return -1;
    }

    const char* name = argv[1];

    int fd = open(name, O_RDONLY);

    if (fd < 0)
    {
        eprintf("Unable to open file `%s`\n", name);
        return -1;
    }

    char* elf_file = (char*)mmap(0, 4096 * 16, PROT_READ | PROT_WRITE, MAP_PRIVATE, fd, 0);

    struct ElfHeader* header = get_elf_header(elf_file);

    printf("Section Headers:\n  [Nr] Name              Type             Address           Offset\n       Size              EntSize          Flags  Link  Info  Align\n");

    char* buffer = malloc(256);

    for (int i = 0; i < header->e_shnum; i++)
    {
        struct SectionHeader* sect = get_section_header(elf_file, i);

        sprintf(buffer, "  [%i]", i);
        tabulate(buffer, 5, true, ' ');

        sprintf(buffer, "  %s", get_section_name(elf_file, i));
        tabulate(buffer, 18, true, ' ');

        sprintf(buffer, "  %s    ", section_type_str(sect->sh_type));
        tabulate(buffer, 19, true, ' ');

        long addr = sect->sh_addr;
        sprintf(buffer, "%x", (int)(addr >> 32));
        tabulate(buffer, 8, false, '0');
        sprintf(buffer, "%x", (int)(addr));
        tabulate(buffer, 8, false, '0');

        printf("  ");

        int offset = sect->sh_offset;
        sprintf(buffer, "%x", offset);
        tabulate(buffer, 8, false, '0');
        
        printf("\n       ");

        long size = sect->sh_size;
        sprintf(buffer, "%x", (int)(size >> 32));
        tabulate(buffer, 8, false, '0');
        sprintf(buffer, "%x", (int)(size));
        tabulate(buffer, 8, false, '0');

        printf("  ");

        long entsize = sect->sh_entsize;
        sprintf(buffer, "%x", (int)(entsize >> 32));
        tabulate(buffer, 8, false, '0');
        sprintf(buffer, "%x", (int)(entsize));
        tabulate(buffer, 8, false, '0');

        // TODO: Add flags, skipping for now
        printf("        ");

        sprintf(buffer, "%x", sect->sh_link);
        tabulate(buffer, 4, false, ' ');

        printf("  ");

        sprintf(buffer, "%x", sect->sh_info);
        tabulate(buffer, 4, false, ' ');

        printf("  ");

        sprintf(buffer, "%x\n", (unsigned int)sect->sh_addralign);
        tabulate(buffer, 4, false, ' ');

        printf("\n");
    }

    free(buffer);

    munmap(elf_file, 4096 * 16);

    close(fd);

    return 0;
}


char* section_type_str(int v)
{
    switch (v)
    {
        case 0:
            return "NULL";
        case 1:
            return "PROGBITS";
        case 2:
            return "SYMTAB";
        case 3:
            return "STRTAB";
        case 4:
            return "RELA";
        case 5:
            return "HASH";
        case 6:
            return "DYNAMIC";
        case 7:
            return "NOTE";
        case 8:
            return "NOBITS";
        case 9:
            return "REL";
        case 0xA:
            return "SHLIB";
        case 0xB:
            return "DYNSYM";
        case 0xE:
            return "INIT_ARRAY";
        case 0xF:
            return "FINI_ARRAY";
        case 0x10:
            return "PREINIT_ARRAY";
        case 0x11:
            return "GROUP";
        case 0x12:
            return "SYMTAB_SHNDX";
        case 0x13:
            return "NUM";
        default:
            return "UNK";
    }
}
