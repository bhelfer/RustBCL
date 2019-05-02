#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]
#![allow(non_snake_case)]

extern crate num;
use num::complex::{Complex, Complex32};

use base::{global_pointer::GlobalPointer, config::Config};
use backend::comm;

use std::time::{SystemTime, Duration};
use std::vec::Vec;
use std::env;
use rand::{rngs::StdRng, Rng, thread_rng, SeedableRng, ChaChaRng};

pub fn benchmark_1d_fft(config: &mut Config) {

    let args: Vec<String> = env::args().collect();
    let n: usize;
    // output debug info or not
    let mut DBG: bool = true;

    if args.len() <= 1 {
        n = 10;
        println!("not enough arguments\nUse default argument n = {}", n);
    } else {
        n = args[1].clone().parse().unwrap();
    }

    if n >= 100 { DBG = false; }

    let rankn: usize = config.rankn as usize;
    let rank: usize = config.rank as usize;
    let mut rng: StdRng = SeedableRng::from_seed([rankn as u8; 32]);

    let size: usize = n * rankn;

    let mut input: GlobalPointer<Complex32> = GlobalPointer::init(config, size);
    if rank == 0 {
        for i in 0 .. size {
            input.idx_rput(
                i as isize,
                Complex::new(rng.gen_range(0, 500), 0)
            );
        }
    }
    comm::barrier();

    /* debug */ if DBG { println!("input = {:?}", input); comm::barrier(); }

    // here start the parallel 1d fft


}