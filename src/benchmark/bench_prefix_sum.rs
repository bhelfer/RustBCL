use global_pointer::GlobalPointer;
use config::Config;
use std::time::{SystemTime, Duration};
use std::vec::Vec;
use comm;

pub fn benchmark_prefix_sum(config: &mut Config) {
    let mut data: Vec<i32> = Vec::new();
    let mut rng: StdRng = SeedableRng::from_seed([rankn as u8; 32]);
    for i in 0 .. n {
        data.push(rng.gen_range(std::i32::MIN, std::i32::MAX));
    }
}