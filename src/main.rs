#![allow(dead_code)]
#![allow(unused)]

mod shmemx;
mod global_pointer;
mod config;
mod comm;
mod array;
mod hash_table;
mod utils;
use config::Config;
use global_pointer::GlobalPointer;
use array::Array;
use hash_table::HashTable;
use utils::Buf;

fn main() {

    let mut config = Config::init(1);
    let rankn = config.rankn;

    if config.rankn < 2 {
        return;
    }

    // ----------- jack's part ------------
    if config.rank == 0 { println!("------------Jack's test------------\n"); }

    let mut ptr1 = GlobalPointer::null();
    if config.rank == 0 {
        let rankn = config.rankn;
        ptr1 = config.alloc::<i32>(rankn);
    }
//    println!("my_rank: {}, ptr before broadcast: {:?}", config.rank, ptr1);
    comm::broadcast(&mut ptr1, 0);
//    println!("my_rank: {}, ptr after broadcast: {:?}", config.rank, ptr1);

    // write value
    (ptr1 + config.rank).rput(config.rank as i32);
    Config::barrier();

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

    Config::barrier();
    // barrier not work TaT, or just println! is slow?
    println!("barrier1, rank{}!", config.rank);
    Config::barrier();

    if config.rank == 1 {
        println!("Rank 1 Sees: ");
        for i in 0..config.rankn {
            value = (ptr1 + i).rget();
            println!("{}: {}", i, value);
        }
    }

    Config::barrier();

    if config.rank == 0 {
        config.free(ptr1);
    }

    // ----------- array's part ------------
    if config.rank == 0 { println!("\n\n------------Array's test------------\n"); }

    let mut arr = Array::<char>::init(&mut config, rankn);
    arr.write(('a' as u8 + config.rank as u8) as char, config.rank);

    Config::barrier();

    if config.rank == 0 {
        for i in 0..config.rankn {
            println!("{}: {}", i, arr.read(i));
        }
    }

    // ----------- HashTable's part ------------
    if config.rank == 0 { println!("\n\n------------HashTable's test------------\n"); }

    let mut hash_table = HashTable::<usize, char>::new(&mut config, 1024);

    let key: usize = config.rank;
    let value  = [char::from('a' as u8 + config.rank as u8), char::from('A' as u8 + config.rank as u8)];

    let mut success = false;

    // Testing for Updating like "hash_table[key] = value"
    for i in 0..2 {
        success = hash_table.insert(&key, &value[i]);
        println!("key is {}, val is {}, insert success = {} by rank {}", key, value[i], success, shmemx::my_pe());
    }

    Config::barrier();

    let mut res: char = '\0';
    for key in 0..(config.rankn + 1) {
        success = hash_table.find(&key, &mut res);
        if success {
            println!("key is {}, find value {:?} by rank {}", key, res, shmemx::my_pe());
        } else {
            println!("key is {}, find nothing by rank {}", key, shmemx::my_pe());
        }
    }

}
