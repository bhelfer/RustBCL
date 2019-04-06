mod global_pointer;

fn main() {
	let mut p1 = global_pointer::GlobalPointer::new(0, 0);
	println!("{:?}", p1);
	println!("{:?}", p1 + 1);
	println!("deref: {}", *p1);
	*p1 = 200;
	println!("deref: {}", *p1);
}