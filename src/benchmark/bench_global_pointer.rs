use base::{global_pointer::GlobalPointer, config::Config};
use backend::comm;

use std::time::{SystemTime, Duration};
use std::vec::Vec;

use statistical;

fn duration_to_nano(duration: &Duration) -> u128 {
    let in_nanos = duration.as_secs() as u128 * 1000_000_000 +
            duration.subsec_nanos() as u128;
    in_nanos
}

pub fn benchmark_global_pointer_remote(config: &mut Config) {
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

    // benchmark code
    let mut data = Vec::new();
    if config.rank == 1 {
        for _ in 0..step {
            let start = SystemTime::now();
            for _ in 0..iter {
                let t = ptr1.rget();
                ptr1.rput(t + 1);
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
        println!("Global Pointer(remote)'s Benchmark: mean: {:.2} nanos, std: {:.2} nanos", mean, standard_deviation);
    }
}

pub fn benchmark_global_pointer_local(config: &mut Config) {
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

    // benchmark code
    let mut data = Vec::new();
    if config.rank == 0 {
        for _ in 0..step {
            let start = SystemTime::now();
            for _ in 0..iter {
                let t = ptr1.rget();
                ptr1.rput(t + 1);
            }
            let duration = SystemTime::now().duration_since(start).unwrap();
            let nanos = duration_to_nano(&duration) / iter as u128;
            data.push(nanos as f64);
        }
    }
    comm::barrier();

    if config.rank == 0 {
    	assert_eq!(ptr1.rget(), iter*step);
        let mean = statistical::mean(&data);
        let standard_deviation = statistical::standard_deviation(&data, None);
        println!("Global Pointer(local)'s Benchmark: mean: {:.2} nanos, std: {:.2} nanos", mean, standard_deviation);
    }
}

pub fn benchmark_global_pointer_local_raw(config: &mut Config) {
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

    // benchmark code
    let mut data = Vec::new();
    if config.rank == 0 {
        for _ in 0..step {
            let start = SystemTime::now();
            for _ in 0..iter {
                let p = ptr1.local_mut();
                unsafe {
                    let t = *p;
                    *p = (t + 1);
                }
            }
            let duration = SystemTime::now().duration_since(start).unwrap();
            let nanos = duration_to_nano(&duration) / iter as u128;
            data.push(nanos as f64);
        }
    }
    comm::barrier();

    if config.rank == 0 {
    	assert_eq!(ptr1.rget(), iter*step);
        let mean = statistical::mean(&data);
        let standard_deviation = statistical::standard_deviation(&data, None);
        println!("Global Pointer(local_raw)'s Benchmark: mean: {:.2} nanos, std: {:.2} nanos", mean, standard_deviation);
    }
}