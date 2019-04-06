use crate::global_pointer::GlobalPointer;

mod global_pointer;

fn main() {
	let mut p1 = GlobalPointer::new(0, 0, 100);
	println!("{:?}", p1);
	println!("{:?}", p1 + 1);
	println!("deref: {}", *p1);
	*p1 = 200;
	println!("deref: {}", *p1);
}