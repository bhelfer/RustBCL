use backend::{comm, shmemx};
use base::{global_pointer::GlobalPointer, config::Config};

use std::time::{SystemTime, Duration};

fn duration_to_nano(duration: &Duration) -> u128 {
    let in_nanos = duration.as_secs() as u128 * 1000_000_000 +
            duration.subsec_nanos() as u128;
    in_nanos
}

pub fn benchmark_shmem(config: &mut Config) {
    let iter: i32 = 100; // iter per step
    let step: i32 = 10000;

    // setup code
    let mut ptr1 = GlobalPointer::null();
    if config.rank == 0 {
        let rankn = config.rankn;
        ptr1 = GlobalPointer::init(config, 1);
        ptr1.rput(0 as i32);
    }
    comm::broadcast(&mut ptr1, 0);
    let raw_ptr = ptr1.rptr() as *mut u8;
    let rank = ptr1.rank;
    let len = 4; //i32
    let mut value = 0;
    let value_ptr = &mut value as *mut i32 as *mut u8;



    // benchmark code
    let mut data = Vec::new();
    if config.rank == 1 {
        for _ in 0..step {
            let start = SystemTime::now();
            for _ in 0..iter {
                unsafe {
                    shmemx::shmem_getmem(value_ptr, raw_ptr, len, rank as i32);
                    value = value + 1;
                    shmemx::shmem_putmem(raw_ptr, value_ptr, len, rank as i32);
                }
            }
            let duration = SystemTime::now().duration_since(start).unwrap();
            let nanos = duration_to_nano(&duration) / iter as u128;
            data.push(nanos as f64);
        }
    }
    comm::barrier();

    if config.rank == 1 {
    	assert_eq!(ptr1.rget(), iter*step);
        let mean = statistical::mean(&data);
        let standard_deviation = statistical::standard_deviation(&data, None);
        println!("Shmem_get/putmem's Benchmark: mean: {:.2} nanos, std: {:.2} nanos", mean, standard_deviation);
    }
}