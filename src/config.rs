use shmemx;
use std::ptr;
use global_pointer::GlobalPointer;
use std::marker::PhantomData;

const SMALLEST_MEM_UNIT: usize = 64; // 64bytes
struct Chunk {
    size: usize,
    last: *mut Chunk,
    next: *mut Chunk
}
/*
----Config----
  Since global mutable variable is a dangerous idea, so I use a struct and pass its reference to
to every where it's needed.
*/
#[derive(Debug, Copy, Clone)]
pub struct Config {
    pub shared_segment_size: usize,
    pub smem_base_ptr: *mut u8, // mut?
    pub rank: usize,
    pub rankn: usize,
    smem_heap: *mut u8,
    flist: *mut Chunk
}

impl Config {
    pub fn init(shared_segment_size_m: usize) -> Config {
        shmemx::init();

        let my_pe = shmemx::my_pe();
        let n_pes = shmemx::n_pes();
        let shared_segment_size = shared_segment_size_m*1024*1024;
        let smem_base_ptr = unsafe{ shmemx::shmem_malloc(shared_segment_size) };
        if smem_base_ptr.is_null() {
            panic!("BCL: Could not allocate shared memory segment.")
        }
        let smem_heap = unsafe{ smem_base_ptr.add(SMALLEST_MEM_UNIT) };// I still don't know why add UNIT

        shmemx::barrier();
        Config {
            shared_segment_size,
            smem_base_ptr,
            rank: my_pe,
            rankn: n_pes,
            smem_heap,
            flist: ptr::null_mut()
        }
    }

    pub fn new_ptr<T>(&self, rank: usize, offset: usize) -> GlobalPointer<T> {
		GlobalPointer{ config: self, rank, offset, refer_type: PhantomData }
	}

    pub fn barrier(self) {
        shmemx::barrier();
    }

    pub fn finalize(self) {
        shmemx::barrier();
        unsafe{shmemx::shmem_free(self.smem_base_ptr as *mut u8)};
        shmemx::finalize();
    }
}