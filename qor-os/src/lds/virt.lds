
OUTPUT_ARCH( "riscv" )


ENTRY( _start )

MEMORY
{
  ram : ORIGIN = 0x80000000, LENGTH = 128M
}


PHDRS
{
  text PT_LOAD;
  data PT_LOAD;
  bss PT_LOAD;
}


SECTIONS
{
  
  .text : {
	 

    PROVIDE(_text_start = .);

    *(.text.init)
	
	*(.eh_frame)

	 *(.text .text.*)
	
    PROVIDE(_text_end = .);
	/
  } >ram AT>ram :text
   
   PROVIDE(_global_pointer = .);
   
  .rodata : {
    PROVIDE(_rodata_start = .);
    *(.rodata .rodata.*)
    PROVIDE(_rodata_end = .);

  } >ram AT>ram :text

  .data : {
	
    . = ALIGN(4096);
    PROVIDE(_data_start = .);

    *(.sdata .sdata.*) *(.data .data.*)
    PROVIDE(_data_end = .);
  } >ram AT>ram :data

  .bss : {
    PROVIDE(_bss_start = .);
    *(.sbss .sbss.*) *(.bss .bss.*)
    PROVIDE(_bss_end = .);
  } >ram AT>ram :bss

  
  PROVIDE(_memory_start = ORIGIN(ram));
  
  PROVIDE(_stack_start = _bss_end);

  . = _stack_start;
  . = . + 0x400000;

  PROVIDE(_stack_end = .);
  PROVIDE(_memory_end = ORIGIN(ram) + LENGTH(ram));

  . = ALIGN(4096);
  PROVIDE(_heap_start = .);
  PROVIDE(_heap_size = _memory_end - _heap_start);
  PROVIDE(_heap_end = _memory_end);
}