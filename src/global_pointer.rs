#![allow(dead_code)]
#![allow(unused)]
use shmemx;
use std::marker::PhantomData;
use std::ops;
use std::mem::size_of;

//pub trait GlobalPointerTrait<T>: ops::Add<isize> + ops::AddAssign<isize> + ops::Sub<isize> + ops::SubAssign<isize> + ops::Index<usize> + ops::IndexMut<usize> + ops::Deref + ops::DerefMut {
//	fn new(rank: usize, ptr: usize) -> Self;
	// fn new(ptr: ptr::null) -> Self;
//}

/*
----GlobalRef----
*/
// to deal with "*p = 1" and "p[0] = 1"
//#[derive(Debug, Copy, Clone)]
//struct GlobalRef<T: GlobalPointable> {
//	ptr: GlobalPointer<T>
//}
//
//impl<T: GlobalPointable> GlobalRef<T> {
//	fn new(p: &GlobalPointer<T>) -> GlobalRef<T> {
//		GlobalRef{ ptr: *p }
//	}
//}

/*
----Config----
  Since global mutable variable is a dangerous idea, so I use a struct and pass its reference to
to every where it's needed.
*/
#[derive(Debug, Copy, Clone)]
pub struct Config {
    shared_segment_size: usize,
    smem_base_ptr: *mut u64, // mut?
    pub rank: usize,
    pub rankn: usize
}

impl Config {
    pub fn init(shared_segment_size_m: usize) -> Config {
        shmemx::init();

        let my_pe = shmemx::my_pe();
        let n_pes = shmemx::n_pes();
        let shared_segment_size = shared_segment_size_m*1024*1024;
        let smem_base_ptr = unsafe{ shmemx::shmem_malloc(shared_segment_size) } as *mut u64;
        // TODO: need check null?

        shmemx::barrier();
        Config {
            shared_segment_size,
            smem_base_ptr,
            rank: my_pe,
            rankn: n_pes,
        }
    }

    pub fn finalize(self) {
        shmemx::barrier();
        unsafe{shmemx::shmem_free(self.smem_base_ptr as *mut u8)};
        shmemx::finalize();
    }
}

/*
----GlobalPointer----
*/
#[derive(Debug, Copy, Clone)]
pub struct GlobalPointer<T> {
    shared_segment_size: usize,
    smem_base_ptr: *mut T,
	rank: usize,
	offset: usize,  // count by size_of(T)
	refer_type: PhantomData<T>
}

// implement GlobalPointer
impl<T> GlobalPointer<T> {
	pub fn new(config: &Config, rank: usize, offset: usize) -> GlobalPointer<T> {
		GlobalPointer{
            shared_segment_size: config.shared_segment_size,
            smem_base_ptr: config.smem_base_ptr as *mut T,
            rank, offset, refer_type: PhantomData }
	}

	pub fn rput(&self, value: T) -> &Self {
        unsafe{ shmemx::shmem_putmem(self.smem_base_ptr.add(self.offset) as *mut u8, &value as *const T as *const u8, size_of::<T>(), self.rank as i32) };

		&self
	}

	// have to get a default value, or "use of possible uninitialized variable"
	pub fn rget(&self, default: T) -> T {
        let mut value: T = default;
        unsafe{ shmemx::shmem_getmem(&mut value as *mut T as *mut u8, self.smem_base_ptr.add(self.offset) as *const u8, size_of::<T>(), self.rank as i32) };
        value
	}
}

//// overload operator+
//impl<T: GlobalPointable> ops::Add<usize> for GlobalPointer<T> {
//    type Output = GlobalPointer<T>;
//
//    fn add(self, n: usize) -> GlobalPointer<T> {
//        GlobalPointer {
//            shared_segment_size,
//            smem_base_ptr,
//        	rank: self.rank,
//        	offset: self.offset + n,
//        	refer_type: PhantomData
//        }
//    }
//}
//
//// overload operator+=
//impl<T: GlobalPointable> ops::AddAssign<usize> for GlobalPointer<T> {
//    fn add_assign(&mut self, n: usize) {
//        self.ptr += n;
//    }
//}
//
//// overload operator[](right)
//impl<T: GlobalPointable> ops::Index<usize> for GlobalPointer<T> {
//    type Output = GlobalRef<T>;
//
//    fn index(&self, i: usize) -> &GlobalRef<T> {
//        &GlobalRef::new(self)
//    }
//}
//
//impl<T: GlobalPointable> ops::IndexMut<usize> for GlobalPointer<T> {
//    fn index_mut(&mut self, i: usize) -> &mut GlobalRef<T> {
//        &mut GlobalRef::new(self)
//    }
//}
//
//impl<T: GlobalPointable> ops::Deref for GlobalPointer<T> {
//    type Target = GlobalRef<T>;
//
//    fn deref(&self) -> &GlobalRef<T> {
//        &GlobalRef::new(self)
//    }
//}
//
//impl<T: GlobalPointable> ops::DerefMut for GlobalPointer<T> {
//    fn deref_mut(&mut self) -> &mut GlobalRef<T> {
//        &mut GlobalRef::new(self)
//    }
//}