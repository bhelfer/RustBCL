#![allow(dead_code)]
#![allow(unused)]
use shmemx;
use std::marker::PhantomData;
use std::ops;
use std::mem::size_of;
use shmemx::shmem_free;
use Config;
use std::ptr;

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
----GlobalPointer----
*/
#[derive(Debug, Copy, Clone)]
pub struct GlobalPointer<'a, T> {
    pub config: &'a Config,
	pub rank: usize,
	pub offset: usize,  // count by size_of(T)
	pub refer_type: PhantomData<T>
}

// implement GlobalPointer
impl<'a, T> GlobalPointer<'a, T> {
    pub fn null() -> GlobalPointer<'static, T> {
        GlobalPointer {
            config: unsafe{ &*ptr::null::<Config>() },
            rank: 0,
            offset: 0,
            refer_type: PhantomData
        }
    }

    pub fn is_null(&self) -> bool {
        let p = self.config as *const Config;
        p.is_null()
    }

    pub fn local(&mut self) -> *mut T {
    	let t = ptr::null_mut::<T>() as *mut T;
        if self.rank != self.config.rank {
            eprintln!("error: calling local() on a remote GlobalPtr");
            return ptr::null_mut::<T>() as *mut T;
        }
        unsafe{ self.config.smem_base_ptr.add(self.offset * size_of::<T>()) as *mut T}
    }

	pub fn rput(&mut self, value: T) -> &mut Self {
        let type_size = size_of::<T>();
        unsafe{ shmemx::shmem_putmem(self.config.smem_base_ptr.add(self.offset * type_size), &value as *const T as *const u8, type_size, self.rank as i32) };

		self
	}

	// have to get a default value, or error "use of possibly uninitialized variable"
	pub fn rget(&self, default: T) -> T {
        let mut value: T = default;
        let type_size = size_of::<T>();
        unsafe{ shmemx::shmem_getmem(&mut value as *mut T as *mut u8, self.config.smem_base_ptr.add(self.offset * type_size), type_size, self.rank as i32) };
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