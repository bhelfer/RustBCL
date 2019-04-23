#![allow(dead_code)]
mod shmemx;
mod global_pointer;
mod config;
mod comm;

use config::Config;
use global_pointer::GlobalPointer;

fn main() {
    let mut config = Config::init(1);

    if config.rankn < 2 {
        return;
    }

    let mut ptr1 = GlobalPointer::null();
    if config.rank == 0 {
	    let rankn = config.rankn;
        ptr1 = config.alloc::<i32>(rankn);
    }
    comm::broadcast(&mut ptr1, 0);

    // test rput, rget
    (ptr1 + config.rank as isize).rput(config.rank as i32);
    comm::barrier();

    let mut value;
    if config.rank == 0 {
        let p1 = ptr1.local();
        let p_slice = unsafe{ std::slice::from_raw_parts(p1, config.rankn) };
        println!("Rank 0 Sees: ");
        for i in 0..config.rankn {
            value = p_slice[i];
            println!("{}: {}", i, value);
        }
    }
    comm::barrier();
    println!("barrier1, rank{}!", config.rank);
    comm::barrier();

    if config.rank == 1 {
        println!("Rank 1 Sees: ");
        for i in 0..config.rankn {
            value = (ptr1 + i as isize).rget();
            println!("{}: {}", i, value);
        }
    }
    comm::barrier();

    // test idx_rget, idx_rput
    ptr1.idx_rput(config.rank as isize, 2*config.rank as i32);
    comm::barrier();

    let mut value;
    if config.rank == 1 {
    	println!("test idx_rget, idx_rput");
        println!("Rank 1 Sees: ");
        for i in 0..config.rankn {
            value = ptr1.idx_rget(i as isize);
            println!("{}: {}", i, value);
        }
    }
    comm::barrier();

    // test arput, arget
    let mut ptr2 = GlobalPointer::null();
    if config.rank == 0 {
        ptr2 = config.alloc::<i32>(6);
        let values = vec![0, 1, 2, 3, 4, 5];
        ptr2.arput(&values);
    }
    comm::broadcast(&mut ptr2, 0);
    if config.rank == 1 {
        println!("test arget, arput");
        let values = ptr2.arget(6);
        println!("rank{}: arget {:?}", config.rank ,values);
    }
}
