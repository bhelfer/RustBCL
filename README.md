# RustBCL
BCL in Rust

## Meeting
#### Apr. 12:
TODO:
- Gptr:
  - impl alloc
  - rput array (rget)
  - pointer arithmetic
- Typersafe view thing:
  - implement slice
  - impl unique/symmetric pointer
- Distributed Data Structures:
  - Array
  - Hash table
  - Queue

## Global Pointer:
#### Apr. 12:
- Done:
  - rearrange and split Config and GlobalPointer;
  - change all memory related value to usize, void\* pointer to \*u8;

#### Apr. 11:
- Done:
  - Config: a struct for "global" variable;
  - GlobalPointer: new, rget, rput;
- TODO:
  - malloc part
  
#### Question
1. why `let smem_heap = smem_base_ptr.add(SMALLEST_MEM_UNIT);`
2. what does the 't' in 'chunk_t' mean?