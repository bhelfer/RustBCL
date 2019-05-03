#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

use backend::shmemx::{self, libc::{c_int, size_t, c_long}};
use base::config::{self, Config, SMALLEST_MEM_UNIT};

use std::marker::PhantomData;
use std::ops;
use std::mem::size_of;
use std::ptr;
use num::complex::{Complex, Complex32, Complex64};

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

pub trait Bclable: Clone + Copy {}
impl Bclable for i8 {}
impl Bclable for i16 {}
impl Bclable for i32 {}
impl Bclable for i64 {}
impl Bclable for isize {}
impl Bclable for u8 {}
impl Bclable for u16 {}
impl Bclable for u32 {}
impl Bclable for u64 {}
impl Bclable for usize {}
impl Bclable for char {}
impl Bclable for Complex32 {}
impl Bclable for Complex64 {}

/*
----GlobalPointer----
*/
#[derive(Debug, Copy, Clone)]
pub struct GlobalPointer<T: Bclable> {
    pub shared_segment_size: usize,
    pub smem_base_ptr: *mut u8,
	pub rank: usize,
	pub offset: usize,  // count by size_of(T)
	pub refer_type: PhantomData<T>
}

// implement GlobalPointer
impl<T: Bclable> GlobalPointer<T> {
    pub fn init(config: &mut Config, mut size: usize) -> GlobalPointer<T> {
        let (_, offset) = config.alloc(size * size_of::<T>());
        
        GlobalPointer {
            shared_segment_size: config.shared_segment_size,
            smem_base_ptr: config.smem_base_ptr,
            rank: config.rank,
            offset,
            refer_type: PhantomData
        }
    }

    pub fn null() -> GlobalPointer<T> {
        GlobalPointer {
            shared_segment_size: 0, // count by bytes
            smem_base_ptr: ptr::null_mut(),
            rank: 0,
            offset: 0,
            refer_type: PhantomData
        }
    }

    pub fn is_null(&self) -> bool {
        self.smem_base_ptr.is_null()
    }

    pub fn local(&self) -> *const T {
    	let t = ptr::null_mut::<T>() as *const T;
        if self.rank != shmemx::my_pe() {
            eprintln!("error: calling local() on a remote GlobalPtr");
            return ptr::null_mut::<T>() as *const T;
        }
        unsafe{
            let p = self.smem_base_ptr.add(self.offset * size_of::<T>()) as *const T;
            assert!(self.is_valid(p as *const u8, size_of::<T>()), "Global Pointer Out of bound!");
            p
        }
    }

    pub fn local_mut(&mut self) -> *mut T {
    	let t = ptr::null_mut::<T>() as *mut T;
        if self.rank != shmemx::my_pe() {
            eprintln!("error: calling local() on a remote GlobalPtr");
            return ptr::null_mut::<T>() as *mut T;
        }
        unsafe{
            let p = self.smem_base_ptr.add(self.offset * size_of::<T>()) as *mut T;
            assert!(self.is_valid(p as *const u8, size_of::<T>()), "Global Pointer Out of bound!");
            p
        }
    }

    pub fn rptr(&mut self) -> *mut T {
        unsafe{
            let p = self.smem_base_ptr.add(self.offset * size_of::<T>()) as *mut T;
            assert!(self.is_valid(p as *const u8, size_of::<T>()), "Global Pointer Out of bound!");
            p
        }
    }

    pub fn rput(&mut self, value: T) -> &mut Self {
        if shmemx::my_pe() == self.rank {
            unsafe {
                *self.local_mut() = value;
            }
        } else {
        	let type_size = size_of::<T>();
	        self.shmem_putmem(unsafe{self.smem_base_ptr.add(self.offset * type_size)}, &value as *const T as *const u8, type_size, self.rank);
        }
		self        
	}

	pub fn rget(&self) -> T {
        if shmemx::my_pe() == self.rank {
            unsafe {
                return *self.local();
            }
        }
        let mut value: T = unsafe{std::mem::uninitialized::<T>()};
        let type_size = size_of::<T>();
        self.shmem_getmem(&mut value as *mut T as *mut u8, unsafe{self.smem_base_ptr.add(self.offset * type_size)}, type_size, self.rank);
        value
	}

    pub fn arput(&mut self, values: &[T]) -> &mut Self {
        let type_size = size_of::<T>();
        self.shmem_putmem(unsafe{self.smem_base_ptr.add(self.offset * type_size)}, values.as_ptr() as *const u8, type_size * values.len(), self.rank);

		self
	}w

	pub fn arget(&self, len: usize) -> Vec<T> {
        let mut values: Vec<T> = vec![unsafe{std::mem::uninitialized::<T>()}; len];
        let type_size = size_of::<T>();
        self.shmem_getmem(values.as_mut_ptr() as *mut u8, unsafe{self.smem_base_ptr.add(self.offset * type_size)}, type_size * len, self.rank);
        values
	}

    pub fn idx_rput(&mut self, idx: isize, value: T) -> &mut Self {
        // (*self + idx).rput(value);
		let type_size = size_of::<T>();
        self.shmem_putmem(unsafe{self.smem_base_ptr.add((self.offset as isize + idx) as usize * type_size)}, &value as *const T as *const u8, type_size, self.rank);
		self
	}

	pub fn idx_rget(&self, idx: isize) -> T {
        // (*self + idx).rget()
		let mut value: T = unsafe{std::mem::uninitialized::<T>()};
        let type_size = size_of::<T>();
        self.shmem_getmem(&mut value as *mut T as *mut u8, unsafe{self.smem_base_ptr.add((self.offset as isize + idx) as usize * type_size)}, type_size, self.rank);
        value
	}

    fn is_valid(&self, p: *const u8, len: usize) -> bool {
        p >= self.smem_base_ptr && unsafe{p.add(len) <= self.smem_base_ptr.add(self.shared_segment_size)}
    }

    fn shmem_putmem(&self, target: *mut u8, source: *const u8, len: usize, pe: usize) {
        assert!(self.is_valid(target, len), "GlobalPtr Out of bound!");
        unsafe{ shmemx::shmem_putmem(target, source, len as size_t, self.rank as c_int) };
    }

    fn shmem_getmem(&self, target: *mut u8, source: *const u8, len: usize, pe: usize) {
        assert!(self.is_valid(source, len), "GlobalPtr Out of bound!");
        unsafe{ shmemx::shmem_getmem(target, source, len as size_t, self.rank as c_int) };
    }
}

// overload operator+
impl<T: Bclable> ops::Add<isize> for GlobalPointer<T> {
    type Output = GlobalPointer<T>;

    fn add(self, n: isize) -> GlobalPointer<T> {
        GlobalPointer {
            shared_segment_size: self.shared_segment_size,
            smem_base_ptr: self.smem_base_ptr,
        	rank: self.rank,
        	offset: (self.offset as isize + n) as usize,
        	refer_type: PhantomData
        }
    }
}

// overload operator+=
impl<T: Bclable> ops::AddAssign<isize> for GlobalPointer<T> {
    fn add_assign(&mut self, n: isize) {
        self.offset = (self.offset as isize + n) as usize;
    }
}

// overload operator+
impl<T: Bclable> ops::Sub<isize> for GlobalPointer<T> {
    type Output = GlobalPointer<T>;

    fn sub(self, n: isize) -> GlobalPointer<T> {
        GlobalPointer {
            shared_segment_size: self.shared_segment_size,
            smem_base_ptr: self.smem_base_ptr,
        	rank: self.rank,
        	offset: (self.offset as isize - n) as usize,
        	refer_type: PhantomData
        }
    }
}

// overload operator+=
impl<T: Bclable> ops::SubAssign<isize> for GlobalPointer<T> {
    fn sub_assign(&mut self, n: isize) {
        self.offset = (self.offset as isize - n) as usize;
    }
}

use std::fmt;

impl<T: Bclable> fmt::Display for GlobalPointer<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.rank, self.offset)
    }
}

// overload operator[](right)
//impl<T> ops::Index<usize> for GlobalPointer<T> {
//    type Output = T;
//
//    fn index(&self, i: usize) -> &T {
//        unsafe{ &((*self + i).rget()) }
//    }
//}

//impl<T: GlobalPointable> ops::IndexMut<usize> for GlobalPointer<T> {
//    fn index_mut(&mut self, i: usize) -> &mut T {
//        &mut GlobalRef::new(self)
//    }
//}

// impl<T> ops::Deref for GlobalPointer<T> {
//     type Target = T;
//
//     fn deref(&self) -> T {
//         self.rget()
//     }
// }

//impl<T: GlobalPointable> ops::DerefMut for GlobalPointer<T> {
//    fn deref_mut(&mut self) -> &mut GlobalRef<T> {
//        &mut GlobalRef::new(self)
//    }
//}
