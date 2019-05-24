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
  
## Algorithms

- TODO:
  - Make more testing, including fine-grained time testing of each subpart. 
  - Find the reason why the scalings on multi-nodes are so terrible.
  - Try to improve the two algorithms.
  
- Done:
  - Distributed Sample Sort
    - Base mostly on GlobalPtr
    - Possible problem: 
      - all-to-all communication with each PE sending $local\_size$ data to each other processor, which is not optimized in communication
      - simpified buffer with many sparing spaces, leading to bad scaling by Amdahl's law
      
  - Distributed Fast Fourier Transformation on 1D (only work when the processor's number is $2^k$, $k \in N$.
    - Based mostly on Array
    - Possible problem: too much blocking in communications
  - Made some scaling tests on Cori
    - Scales not too bad on 1 node, multi-cores.
    - Scales extremely terrible on multi-nodes.
  
## HashTable
  

#### Apr. 20:
- Done:
  - Add test.sh, used to genereate test in `[test]` to binary and run it with `oshrun`.
  - need **jq** to find the binary (you can `sudo apt-get install jq`)
  - Test Not Passed Yet on docker
    - That's because the Openshmem is not implemented well on the docker version. The version on Cori is robust to millions of operations. No problem now.
    
#### Apr. 19:
- Done:
  - use `Config::alloc()` for initializing
  - implement only for `key: K`, `value: V` where K and V impl specific traits 
  - implement atomic `HashTable<K, V>::insert()`. It will update `value: V` if `key: K` is inserted before
  - implement `HashTable<K, V>::find()`

## Global Pointer:
#### TODO?
- null->option

#### Apr. 23:
- Done:
  - Alloc: Memory Align

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
  
## SafeBCL 
#### Apr. 26:
- Done:
  - GlobalGuard
  - GlobalValue



#### Question
1. why `let smem_heap = smem_base_ptr.add(SMALLEST_MEM_UNIT);`
- Where is it from? 
2. what does the 't' in 'chunk_t' mean?
