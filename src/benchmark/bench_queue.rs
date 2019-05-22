use base::config::Config;
use backend::comm;
use benchmark::tools::duration_to_nano;

use containers::queue::Queue;

use std::time::SystemTime;
use rand::{
	rngs::StdRng,
	Rng,
	SeedableRng
};
use std::fmt;

pub fn test_rand(config: &mut Config, total_workload: usize) {
//    let total_workload = 131072;
    let local_workload = (total_workload + config.rankn - 1) / config.rankn;

    let seed: [u8; 32] = [1; 32];
    let mut rng = StdRng::from_seed(seed);
    let mut evl_rng = StdRng::from_seed(seed);
    // test correctness
    for _ in 0..local_workload {
        let evl_value: u32 = evl_rng.gen();
        let value: u32 = rng.gen();
        assert_eq!(value, evl_value, "rand correctness fail!");
    }
    comm::barrier();
    if config.rank == 0 {
        println!("rand correctness pass!");
    }
}


pub fn benchmark_queue(config: &mut Config, total_workload: usize, label: &str) {
//    let total_workload = 131072;
    let local_workload = (total_workload + config.rankn - 1) / config.rankn;

    let mut queue = Queue::new(config, total_workload as usize);
    let seed: [u8; 32] = [1; 32];
    let mut rng = StdRng::from_seed(seed);

    // benchmark push
    comm::barrier();
    let start = SystemTime::now();
    for _ in 0..local_workload {
        let value: u32 = rng.gen();
        queue.push(value);
    }
    comm::barrier();
    let push_duration = SystemTime::now().duration_since(start).expect("SystemTime::duration_since failed");
    let push_nanos = duration_to_nano(&push_duration);

    let mut evl_rng = StdRng::from_seed(seed);
    // benchmark pull
    comm::barrier();
    let start = SystemTime::now();
    for i in 0..local_workload {
        let evl_value: u32 = evl_rng.gen();
        let f = queue.pop();
        match f {
            Err(error) => println!("{}", error),
            Ok(value) => assert_eq!(value, evl_value, "{}: Queue assert error!", i),
        }
    }
    let pop_duration = SystemTime::now().duration_since(start).expect("SystemTime::duration_since failed");
    let pop_nanos = duration_to_nano(&pop_duration);

    // print out result
    if config.rank == 0 {
        println!("Queue {}: {}, {}, {}", label, config.rankn, push_nanos, pop_nanos);
    }
}
