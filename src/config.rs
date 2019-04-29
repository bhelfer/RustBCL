#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

use shmemx;
use std::ptr;
use global_pointer::GlobalPointer;
use std::marker::PhantomData;
use std::mem::size_of;
use std::io::{stdout, Write};

// simple alloc doesn't need these things
pub const SMALLEST_MEM_UNIT: usize = 64; // 64bytes
// #[derive(Debug, Copy, Clone)]
// struct Chunk {
//     size: usize,
//     last: *mut Chunk,
//     next: *mut Chunk
// }
/*
----Config----
  Since global mutable variable is a dangerous idea, so I use a struct and pass its reference to
to every where it's needed.
*/
#[derive(Debug)]
pub struct Config {
    pub shared_segment_size: usize,
    pub smem_base_ptr: *mut u8,
    pub rank: usize,
    pub rankn: usize,
    smem_heap: *mut u8,
//    flist: *mut Chunk // naive malloc doesn't need this.
}

impl Config {
    pub fn init(shared_segment_size_m: usize) -> Self {
        shmemx::init();

        let my_pe = shmemx::my_pe();
        let n_pes = shmemx::n_pes();
        let shared_segment_size = shared_segment_size_m*1024*1024;
        let smem_base_ptr = unsafe{ shmemx::shmem_malloc(shared_segment_size) };
        if smem_base_ptr.is_null() {
            panic!("BCL: Could not allocate shared memory segment.")
        }
        let smem_heap = unsafe{ smem_base_ptr.add(SMALLEST_MEM_UNIT) };

        shmemx::barrier();
        Self {
            shared_segment_size, // count by bytes
            smem_base_ptr,
            rank: my_pe,
            rankn: n_pes,
            smem_heap,
        }
    }

    pub unsafe fn new_ptr<T>(&self, rank: usize, offset: usize) -> GlobalPointer<T> {
		GlobalPointer{ 
			shared_segment_size: self.shared_segment_size, 
			smem_base_ptr: self.smem_base_ptr,
			rank, 
			offset, 
			refer_type: PhantomData 
		}
	}

    // changed to global method by lfz
    pub fn barrier() {
    	println!("Config::barrier: change to comm::barrier");
        shmemx::barrier();
    }

    fn finalize(&self) {
        shmemx::barrier();
        unsafe{shmemx::shmem_free(self.smem_base_ptr as *mut u8)};
        shmemx::finalize();
    }

    // malloc part
    // size: byte size
    pub fn alloc(&mut self, mut raw_size: usize) -> (*mut u8, usize) {
        let size = ((raw_size + SMALLEST_MEM_UNIT - 1) / SMALLEST_MEM_UNIT) * SMALLEST_MEM_UNIT; // align size

        // if we have run out of heap...
        unsafe {
            if self.smem_heap.add(size) > self.smem_base_ptr.add(self.shared_segment_size) {
            	panic!("run out of symmetrical memory");
                // return (ptr::null(), 0);
            }
        }

        let allocd: *mut u8 = self.smem_heap as *mut u8;
        unsafe{ self.smem_heap = self.smem_heap.add(size); }
       	// println!("Rank {} alloc memory! smem_base_ptr: {:p}, smem_heap: {:p}, allocd: {:p}, raw_size: {}bytes, size: {}bytes", self.rank, self.smem_base_ptr, self.smem_heap, allocd, raw_size, size);
        (allocd, allocd as usize - self.smem_base_ptr as usize)
    }

    pub fn free<T>(&mut self, p: GlobalPointer<T>) {
        // dull free does nothing
    }
}

// deconstructor
impl Drop for Config {
    fn drop(&mut self) {
        self.finalize()
    }
}