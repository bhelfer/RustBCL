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

#### Apr. 19:
TODO:
- Benchmarks:
  - micro-benchmarks
  - simple algorithms like `QuickSort`
  
- Distributed Data Structures:
  - Array: slice
  - Hash table: need to be refined
  - Queue
  
## HashTable

#### TODO?
  Test Not Passed Yet!!!
  Do not know why PE will stuck in functions like `rget()`.
  
#### Apr. 19:
- Done:
  - use `Config::alloc()` for initializing
  - implement only for `key: K`, `value: V` where K and V impl specific traits 
  - implement atomic `HashTable<K, V>::insert()`. It will update `value: V` if `key: K` is inserted before
  - implement `HashTable<K, V>::find()`

## Global Pointer:
#### TODO?
- null->option

#### Apr. 22:
- Done:
  - arget, arput, idx_rget, idx_rput
  - config::barrier -> comm::barrier

#### Apr. 19:
- Done:
  - fix bugs of `self` parameter
  - Change `Config::barrier()` to global methods
  
#### Apr. 13:
- Done:
  - simple alloc; broadcast; ops(add, sub, add/sub assign)
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
- Where is it from? 
2. what does the 't' in 'chunk_t' mean?
