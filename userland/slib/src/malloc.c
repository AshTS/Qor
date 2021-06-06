#include "malloc.h"
#include "printf.h"
#include "syscalls.h"

#define PAGESIZE 4096
#define INITIAL_HEAP 4096

#ifdef DEBUG
    #define DEBUG_PRINT(...) printf(__VA_ARGS__)
#else
    #define DEBUG_PRINT(...)
#endif

static void* heap_start = 0;
static int heap_size = 0;

static void* heap_table = 0;
static int heap_table_size = 0;

#define CHUNK_VALID 4
#define CHUNK_FREE 2

typedef struct MallocChunk
{
    void* ptr;
    unsigned long size;
    struct MallocChunk* next;
    char flags;
} MallocChunk;

// Get the next available chunk in the 
MallocChunk* next_chunk()
{
    MallocChunk* walk = (MallocChunk*)heap_table;

    while ((void*)walk - heap_table < heap_table_size)
    {
        if ((walk->flags & CHUNK_VALID) == 0)
        {
            return walk;
        }

        walk++;
    }

    printf("No more malloc chunks available!");
    return 0;
}

void* malloc(unsigned int size)
{
    DEBUG_PRINT("malloc(%i) => ", size);
    if (heap_start == 0)
    {
        heap_start = mmap(0, INITIAL_HEAP, PROT_READ | PROT_WRITE, MAP_ANONYMOUS, 0, 0);
        heap_size = INITIAL_HEAP;

        heap_table = mmap(0, PAGESIZE, PROT_READ | PROT_WRITE, MAP_ANONYMOUS, 0, 0);
        heap_table_size = PAGESIZE;

        MallocChunk* walk = (MallocChunk*)heap_table;

        walk->ptr = heap_start;
        walk->size = heap_size;
        walk->flags = CHUNK_FREE | CHUNK_VALID;
        walk->next = 0;


        walk++;

        while ((void*)walk - heap_table < heap_table_size)
        {
            walk->flags = 0;

            walk++;
        }
    }

    MallocChunk* walk = (MallocChunk*)heap_table;
    void* result = 0;

    while (walk != 0 && (void*)walk - heap_table < heap_table_size)
    {
        if ((walk->flags & CHUNK_VALID) == 0)
        {
            printf("Malloc Hit an Invalid Chunk\n");
            exit(-1);
        }

        if (walk->flags & CHUNK_FREE)
        {
            if (walk->size >= size)
            {
                result = walk->ptr;
                walk->flags ^= CHUNK_FREE;
                
                if (walk->size != size)
                {
                    MallocChunk* next = next_chunk();

                    next->flags = CHUNK_FREE | CHUNK_VALID;
                    next->size = walk->size - size;
                    next->ptr = walk->ptr + size;

                    walk->size = size;

                    next->next = walk->next;
                    walk->next = next;
                }
                break;
            }
        }

        walk = walk->next;
    }

    DEBUG_PRINT("%p\n", result);
    return result;
}

void flush_helper(unsigned int* count, MallocChunk** first)
{
    if (*count > 0)
    {
        unsigned long add_size = 0;

        MallocChunk* size_walk = *first;


        if (size_walk->next != 0)
        {
            while ((size_walk->next)->flags & CHUNK_FREE)
            {
                size_walk = size_walk->next;
                add_size += size_walk->size;

                if(size_walk->next == 0) { break; }
            }

            (*first)->size += add_size;
            (*first)->next = size_walk->next;
        }
    }
    
    *first = 0;
    *count = 0;
}

void free(void* ptr)
{
    DEBUG_PRINT("free(%p)\n", ptr);

    MallocChunk* walk = (MallocChunk*)heap_table;
    MallocChunk* first = 0;
    unsigned int count = 0;
    unsigned int expected_next = 0;

    while (walk != 0 && (void*)walk - heap_table < heap_table_size)
    {
        if ((walk->flags & CHUNK_VALID) == 0)
        {
            printf("Malloc Hit an Invalid Chunk\n");
            exit(-1);
        }

        if (!(walk->flags & CHUNK_FREE))
        {
            if (walk->ptr == ptr)
            {
                walk->flags |= CHUNK_FREE;
            }
        }

        char flush = 1;

        if ((walk->flags & CHUNK_FREE))
        {
            if (first == 0)
            {
                flush = 0;
                first = walk;
                count = 1;
            }
            else if (walk->ptr != expected_next)
            {
                flush = 0;
                count++;
            }

            expected_next = walk->ptr + walk->size;
        }

        if (flush)
        {
            flush_helper(&count, &first);
        }
        
        walk = walk->next;
    }

    flush_helper(&count, &first);
}

#ifdef DEBUG
void dump()
{
    if (heap_table == 0)
    {
        printf("Heap not initialized\n");
        return;
    }

    MallocChunk* walk = (MallocChunk*)heap_table;

    while (walk != 0 && (void*)walk - heap_table < heap_table_size)
    {
        
        if ((walk->flags & CHUNK_VALID) == 0)
        {
            printf("Invalid Chunk\n");
            break;
        }

        if ((walk->flags & CHUNK_FREE) == 0)
        {
            printf("[ALLOC] ");
        }
        else
        {
            printf("[FREE ] ");
        }

        printf("%p %ld byte%c\n", walk->ptr, walk->size, (walk->size > 1) ? 's' : ' ');

        walk = walk->next;
    }
}
#else
void dump()
{
    
}
#endif