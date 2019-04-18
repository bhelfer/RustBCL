#![allow(dead_code)]
mod shmemx;
mod global_pointer;
mod config;
mod comm;
mod array;
use config::Config;
use global_pointer::GlobalPointer;
use array::Array;
fn main() {
    let mut config = Config::init(1);
	let rankn = config.rankn;

    if config.rankn < 2 {
        config.finalize();
        return;
    }
    let mut ptr1 = GlobalPointer::null();
    if config.rank == 0 {
        ptr1 = config.alloc::<i32>(rankn);
    }
    println!("my_rank: {}, ptr before broadcast: {:?}", config.rank, ptr1);
    comm::broadcast(&mut ptr1, 0);
    println!("my_rank: {}, ptr after broadcast: {:?}", config.rank, ptr1);

    // write value
    (ptr1 + config.rank).rput(config.rank as i32);
    config.barrier();

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
    config.barrier();
    // barrier not work TaT, or just println! is slow?
    println!("barrier1, rank{}!", config.rank);
    config.barrier();

    if config.rank == 1 {
        println!("Rank 1 Sees: ");
        for i in 0..config.rankn {
            value = (ptr1 + i).rget();
            println!("{}: {}", i, value);
        }
    }
    config.barrier();

    let mut arr = Array::<char>::init(&mut config, rankn);
//    arr.array(100);  // what does this method do?
    arr.write(('a' as u8 + config.rank as u8) as char, config.rank);
    config.barrier();
    if config.rank == 0 {
        for i in 0..config.rankn {
            println!("{}: {}", i, arr.read(i));
        }
    }

    if config.rank == 0 {
        config.free(ptr1);
    }
    config.finalize();
}
