#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

use backend::shmemx::{self, libc::{self, c_int, size_t, c_long, c_void}};
use base::config::{self, Config, SMALLEST_MEM_UNIT};

use std::marker::PhantomData;
use std::ops;
use std::mem::size_of;
use std::ptr;
use std::fmt;

/*
----Bclable----
*/
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

/*
----GlobalPointer----
*/
#[derive(Debug, Copy, Clone)]
pub struct GlobalPointer<T: Bclable> {
	pub rank: usize,
	pub ptr: *mut u8,
	pub refer_type: PhantomData<T>
}

// implement GlobalPointer
impl<T: Bclable> GlobalPointer<T> {
    pub fn init(config: &mut Config, mut size: usize) -> GlobalPointer<T> {
        let (ptr, _) = config.alloc(size * size_of::<T>());
        
        GlobalPointer {
            rank: config.rank,
            ptr,
            refer_type: PhantomData
        }
    }

    pub fn null() -> GlobalPointer<T> {
        GlobalPointer {
            rank: 0,
            ptr: ptr::null_mut() as *mut u8,
            refer_type: PhantomData
        }
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    pub fn local(&self) -> *const T {
    	let t = ptr::null_mut::<T>() as *const T;
        if self.rank != shmemx::my_pe() {
            eprintln!("error: calling local() on a remote GlobalPtr");
            return ptr::null_mut::<T>() as *const T;
        }
        unsafe{
        	self.ptr as *const T
        }
    }

    pub fn local_mut(&mut self) -> *mut T {
    	let t = ptr::null_mut::<T>() as *mut T;
        if self.rank != shmemx::my_pe() {
            eprintln!("error: calling local() on a remote GlobalPtr");
            return ptr::null_mut::<T>() as *mut T;
        }
        unsafe{
        	self.ptr as *mut T
        }
    }

    pub fn remote(&self) -> *const T {
        unsafe{
            self.ptr as *const T
        }
    }

    pub fn remote_mut(&mut self) -> *mut T {
        unsafe{
            self.ptr as *mut T
        }
    }

    pub fn rput(&mut self, value: T) -> &mut Self {
    	unsafe {
        	let type_size = size_of::<T>();
	        self.putmem(self.ptr, &value as *const T as *const u8, type_size, self.rank);
			self   
    	}
	}

	pub fn rget(&self) -> T {
		unsafe {
	        let mut value: T = unsafe{std::mem::uninitialized::<T>()};
	        let type_size = size_of::<T>();
	        self.getmem(&mut value as *mut T as *mut u8, self.ptr, type_size, self.rank);
	        value
		}
	}

    pub fn arput(&mut self, values: &[T]) -> &mut Self {
    	unsafe {
    		let type_size = size_of::<T>();
	        self.putmem(self.ptr, values.as_ptr() as *const u8, type_size * values.len(), self.rank);
			self
    	}
	}

	pub fn arget(&self, len: usize) -> Vec<T> {
		unsafe {
			let mut values: Vec<T> = vec![unsafe{std::mem::uninitialized::<T>()}; len];
        	let type_size = size_of::<T>();
        	self.getmem(values.as_mut_ptr() as *mut u8, self.ptr, type_size * len, self.rank);
        	values
		}
	}

    pub fn idx_rput(&mut self, idx: isize, value: T) -> &mut Self {
        // (*self + idx).rput(value);
        unsafe {
        	let type_size = size_of::<T>();
	        self.putmem(self.ptr.add(idx as usize * type_size), &value as *const T as *const u8, type_size, self.rank);
			self
        }
	}

	pub fn idx_rget(&self, idx: isize) -> T {
        // (*self + idx).rget()
        unsafe {
			let mut value: T = unsafe{std::mem::uninitialized::<T>()};
	        let type_size = size_of::<T>();
	        self.getmem(&mut value as *mut T as *mut u8, self.ptr.add(idx as usize * type_size), type_size, self.rank);
	        value        	
        }
	}

    unsafe fn putmem(&self, target: *mut u8, source: *const u8, len: usize, pe: usize) {
    	if shmemx::my_pe() == self.rank {
    		libc::memcpy(target as *mut c_void, source as *const T as *const c_void, len * size_of::<T>());
    	} else {
	        shmemx::shmem_putmem(target, source, len as size_t, self.rank as c_int);
    	}
    }

    unsafe fn getmem(&self, target: *mut u8, source: *const u8, len: usize, pe: usize) {
    	if shmemx::my_pe() == self.rank {
    		libc::memcpy(target as *mut c_void, source as *const T as *const c_void, len * size_of::<T>());
    	} else {
     	   shmemx::shmem_getmem(target, source, len as size_t, self.rank as c_int);
    	}
    }
}

// overload operator+
impl<T: Bclable> ops::Add<isize> for GlobalPointer<T> {
    type Output = GlobalPointer<T>;

    fn add(self, n: isize) -> GlobalPointer<T> {
    	unsafe {
    		if n >= 0 {
    			GlobalPointer {
		        	rank: self.rank,
		        	ptr: self.ptr.add(n as usize * size_of::<T>()),
		        	refer_type: PhantomData
		        }
    		} else {
    			GlobalPointer {
		        	rank: self.rank,
		        	ptr: self.ptr.sub(n as usize * size_of::<T>()),
		        	refer_type: PhantomData
		        }
    		}
    	}
    }
}

// overload operator+=
impl<T: Bclable> ops::AddAssign<isize> for GlobalPointer<T> {
    fn add_assign(&mut self, n: isize) {
    	unsafe {
    		if n >= 0 {
    			self.ptr = self.ptr.add(n as usize * size_of::<T>())
    		} else {
    			self.ptr = self.ptr.sub(n as usize * size_of::<T>())
    		}
    	}
    }
}

// overload operator+
impl<T: Bclable> ops::Sub<isize> for GlobalPointer<T> {
    type Output = GlobalPointer<T>;

    fn sub(self, n: isize) -> GlobalPointer<T> {
        unsafe {
    		if n <= 0 {
    			GlobalPointer {
		        	rank: self.rank,
		        	ptr: self.ptr.add(n as usize * size_of::<T>()),
		        	refer_type: PhantomData
		        }
    		} else {
    			GlobalPointer {
		        	rank: self.rank,
		        	ptr: self.ptr.sub(n as usize * size_of::<T>()),
		        	refer_type: PhantomData
		        }
    		}
    	}
    }
}

// overload operator+=
impl<T: Bclable> ops::SubAssign<isize> for GlobalPointer<T> {
    fn sub_assign(&mut self, n: isize) {
        unsafe {
    		if n <= 0 {
    			self.ptr = self.ptr.add(n as usize * size_of::<T>())
    		} else {
    			self.ptr = self.ptr.sub(n as usize * size_of::<T>())
    		}
    	}
    }
}


impl<T: Bclable> fmt::Display for GlobalPointer<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {:p})", self.rank, self.ptr)
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
