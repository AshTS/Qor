#include "libelf.h"

// Get a pointer to the ELF header of the file buffer given
struct ElfHeader* get_elf_header(void* file)
{
    return (struct ElfHeader*)file;
}

// Get a pointer to the section header with the given index, or a null pointer
//   if such a section header does not exist.
struct SectionHeader* get_section_header(void* file, unsigned int index)
{
    struct ElfHeader* header = (struct ElfHeader*)file;

    if (index >= header->e_shnum)
    {
        return 0;
    }
    else
    {
        return (struct SectionHeader*)(file + header->e_shoff) + index;
    }
}

// Get a pointer to the contents of the section with the given index
void* get_section_contents(void* file, unsigned int index)
{
    struct ElfHeader* header = (struct ElfHeader*)file;

    struct SectionHeader* section = get_section_header(file, index);

    if (section == 0)
    {
        return 0;
    }

    return file + section->sh_offset;
}

// Get a C String representation of the name of the section with the given index
char* get_section_name(void* file, unsigned int index)
{
    struct ElfHeader* header = (struct ElfHeader*)file;
    int str_index = header->e_shstrndx;

    struct SectionHeader* section = get_section_header(file, index);

    if (section == 0)
    {
        return 0;
    }

    char* buffer = get_section_contents(file, str_index);

    if (buffer == 0)
    {
        return 0;
    }

    return buffer + section->sh_name;
}

