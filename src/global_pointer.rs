use std::ops;
use std::ptr;

trait GlobalRefTrait<T> {
	fn assign(value: T);
}

struct GlobalRef<T> {

}

pub trait GlobalPointerTrait<T>: ops::Add<isize> + ops::AddAssign<isize> + ops::Sub<isize> + ops::SubAssign<isize> + ops::Index<usize> + ops::IndexMut<usize> + ops::Deref + ops::DerefMut {
	fn new(rank: usize, ptr: usize) -> Self;
	// fn new(ptr: ptr::null) -> Self;

}

#[derive(Debug)]
pub struct GlobalPointer<T> {
	rank: usize,
	ptr: usize,
	fake_value: T
}

impl<T> GlobalPointer<T> {
	pub fn new(rank: usize, ptr: usize, value: T) -> GlobalPointer<T> {
		GlobalPointer {
        	rank: rank,
        	ptr: ptr,
        	fake_value: value
        }
	}

	fn derefer(&self, i: usize) -> &T {
		&self.fake_value
	}

	fn dereferMut(&mut self, i: usize) -> &mut T {
		&mut self.fake_value
	}
}

impl<T> ops::Add<usize> for GlobalPointer<T> {
    type Output = GlobalPointer<T>;

    fn add(self, n: usize) -> GlobalPointer<T> {
        GlobalPointer {
        	rank: self.rank,
        	ptr: self.ptr + n,
        	fake_value: self.fake_value
        }
    }
}

impl<T> ops::AddAssign<usize> for GlobalPointer<T> {
    fn add_assign(&mut self, n: usize) {
        *self = GlobalPointer {
        	rank: self.rank,
        	ptr: self.ptr + n,
        	fake_value: self.fake_value
        }
    }
}

impl<T> ops::Index<usize> for GlobalPointer<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        self.derefer(i)
    }
}

impl<T> ops::IndexMut<usize> for GlobalPointer<T> {
    fn index_mut(&mut self, i: usize) -> &mut GlobalRef<T> {
        self.dereferMut(i)
    }
}

impl<T> ops::Deref for GlobalPointer<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.derefer(0)
    }
}

impl<T> ops::DerefMut for GlobalPointer<T> {
    fn deref_mut(&mut self) -> &mut GlobalRef<T> {
        self.dereferMut(0)
    }
}