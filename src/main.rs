#![allow(dead_code)]
#![allow(unused)]

mod shmemx;
mod global_pointer;
mod config;
mod comm;
mod array;
mod hash_table;
use config::Config;
use global_pointer::GlobalPointer;
use array::Array;
use hash_table::HashTable;

fn main() {
    let mut config = Config::init(1);
    let rankn = config.rankn;

    if config.rankn < 2 {
        config.finalize();
        return;
    }

    // array part
//    let mut arr = Array::<char>::init(&mut config, rankn);
//    arr.write(('a' as u8 + config.rank as u8) as char, config.rank);
//    config.barrier();
//    if config.rank == 0 {
//        for i in 0..config.rankn {
//            println!("{}: {}", i, arr.read(i));
//        }
//    }


    // hashtable part
    let mut hash_table = HashTable::<usize, char>::new(&mut config, 1024);
    config.barrier();

    let key: usize = config.rank * 11;
    let value: char = char::from('a' as u8 + config.rank as u8);
//        if config.rank == 0 {'a'} else {'b'};

    let mut success = hash_table.insert(&key, &value);
    config.barrier();
    println!("insert success = {} on rank {}", success, shmemx::my_pe());

    let mut res: char = '\0';
    success = hash_table.find(&key, &mut res);
    if success {
        println!("insert value {:?}, find value {:?} on rank {}", value, res, shmemx::my_pe());
    } else {
        println!("insert value {:?}, find nothing on rank {}", value, shmemx::my_pe());
    }

    config.finalize();
}
