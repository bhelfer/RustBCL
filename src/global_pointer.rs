use std::marker::PhantomData;

use std::ops;

trait GlobalPointable: Clone{}

trait GlobalRefTrait<T> {
	fn assign(value: T);
}

pub trait GlobalPointerTrait<T>: ops::Add<isize> + ops::AddAssign<isize> + ops::Sub<isize> + ops::SubAssign<isize> + ops::Index<usize> + ops::IndexMut<usize> + ops::Deref + ops::DerefMut {
	fn new(rank: usize, ptr: usize) -> Self;
	// fn new(ptr: ptr::null) -> Self;
}

/*
----GlobalRef----
*/
// to deal with "*p = 1" and "p[0] = 1"
#[derive(Debug, Copy, Clone)]
struct GlobalRef<T: GlobalPointable> {
	ptr: GlobalPointer<T>
}

impl<T: GlobalPointable> GlobalRef<T> {
	fn new(p: &GlobalPointer<T>) -> GlobalRef<T> {
		GlobalRef{ ptr: *p }
	}
}

/*
----GlobalPointer----
*/
#[derive(Debug, Copy, Clone)]
pub struct GlobalPointer<T: GlobalPointable> {
	rank: usize,
	ptr: usize,
	refer_type: PhantomData<T>
}

// implement GlobalPointer
impl<T: GlobalPointable> GlobalPointer<T> {
	pub fn new(rank: usize, ptr: usize) -> GlobalPointer<T> {
		GlobalPointer{ rank, ptr, refer_type: PhantomData }
	}

	pub fn assign(&self, value: T) -> &Self {
		&self
	}

	pub fn deref(&self) -> T {

	}
}

// overload operator+
impl<T: GlobalPointable> ops::Add<usize> for GlobalPointer<T> {
    type Output = GlobalPointer<T>;

    fn add(self, n: usize) -> GlobalPointer<T> {
        GlobalPointer {
        	rank: self.rank,
        	ptr: self.ptr + n,
        	refer_type: PhantomData
        }
    }
}

// overload operator+=
impl<T: GlobalPointable> ops::AddAssign<usize> for GlobalPointer<T> {
    fn add_assign(&mut self, n: usize) {
        self.ptr += n;
    }
}

// overload operator[](right)
impl<T: GlobalPointable> ops::Index<usize> for GlobalPointer<T> {
    type Output = GlobalRef<T>;

    fn index(&self, i: usize) -> &GlobalRef<T> {
        &GlobalRef::new(self)
    }
}

impl<T: GlobalPointable> ops::IndexMut<usize> for GlobalPointer<T> {
    fn index_mut(&mut self, i: usize) -> &mut GlobalRef<T> {
        &mut GlobalRef::new(self)
    }
}

impl<T: GlobalPointable> ops::Deref for GlobalPointer<T> {
    type Target = GlobalRef<T>;

    fn deref(&self) -> &GlobalRef<T> {
        &GlobalRef::new(self)
    }
}

impl<T: GlobalPointable> ops::DerefMut for GlobalPointer<T> {
    fn deref_mut(&mut self) -> &mut GlobalRef<T> {
        &mut GlobalRef::new(self)
    }
}