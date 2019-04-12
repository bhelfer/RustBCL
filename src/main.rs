#![allow(dead_code)]
mod shmemx;
mod global_pointer;

use global_pointer::{Config, GlobalPointer};

fn main() {
    let config = Config::init(1);

    if config.rankn < 2 {
        config.finalize();
        return;
    }
    let ptr1 = GlobalPointer::new(&config, 0, 1);
//    let ptr2 = GlobalPointer::new(&config, 1, 1);

    if config.rank == 1 {
        ptr1.rput(1);
    }
    if config.rank == 0 {
        let value = ptr1.rget(0);
        println!("rget: {}", value);
    }

    config.finalize();
}
