#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

extern crate is_sorted;
extern crate quickersort;

use base::{global_pointer::GlobalPointer, config::Config};
use backend::comm;

use std::time::{SystemTime, Duration};
use std::vec::Vec;
use std::env;
use rand::{rngs::StdRng, Rng, thread_rng, SeedableRng, ChaChaRng};
use is_sorted::IsSorted;
use backend::comm::gather;

pub fn benchmark_sample_sort(config: &mut Config) {

    let args: Vec<String> = env::args().collect();
    let n: usize;

    if args.len() <= 1 {
        n = 10;
        println!("not enough arguments\nUse default argument n = {}", n);
    } else {
        n = args[1].clone().parse().unwrap();
    }


    let rankn: usize = config.rankn as usize;
    let rank: usize = config.rank as usize;
    let mut rng: StdRng = SeedableRng::from_seed([rankn as u8; 32]);

    let size: usize = n * rankn;

    let mut data: GlobalPointer<u32> = GlobalPointer::init(config, size);
    if rank == 0 {
        for i in 0 .. size {
            data.idx_rput(
                i as isize,
                rng.gen_range(0, 500)
            );
        }
    }
    comm::barrier();

    /* debug */ { println!("data = {:?}", data); comm::barrier(); }

    // here start the sorting

    // scattering data from rank 0
    let mut loc_data: Vec<GlobalPointer<u32>> = Vec::new();
    loc_data.resize(rankn as usize, GlobalPointer::null());
    loc_data[rank] = GlobalPointer::init(config, n as usize);
    comm::barrier();

    comm::scatter(&mut loc_data[rank], &mut data, 0, n);
    comm::barrier();

    // local sorting(user serial type)
    let mut loc_data_serial = loc_data[rank].arget(n);
    quickersort::sort(&mut loc_data_serial[..]);
    comm::barrier();

    /* debug */ {
        assert_eq!(IsSorted::is_sorted(&mut loc_data_serial.iter()), true);
        println!("rank {}: {:?}", rank, loc_data_serial); comm::barrier();
    }

    // choosing local pivots
    let mut loc_pivots: Vec<GlobalPointer<u32>> = Vec::new();
    loc_pivots.resize(rankn, GlobalPointer::null());
    loc_pivots[rank] = GlobalPointer::init(config, rankn - 1);
    for i in 0 .. rankn - 1 {
        loc_pivots[rank].idx_rput(
            i as isize,
            loc_data_serial[size / (rankn * rankn) * (i + 1)]
        );
    }
    comm::barrier();

    // gather local pivots to rank 0
    let mut pivots: GlobalPointer<u32> = GlobalPointer::init(config, rankn * (rankn - 1));
    comm::gather(&mut pivots, &mut loc_pivots[rank], 0, rankn - 1);
    comm::barrier();

    // sort all pivots in rank 0
    if rank == 0 {
        let mut pivots_serial = pivots.arget(rankn * (rankn - 1));
        quickersort::sort(&mut pivots_serial[..]);

        /* debug */ {  println!("root is rank {}: {:?}", rank, pivots_serial); }
        for i in 0 .. rankn - 1 {
            loc_pivots[0].idx_rput(
                i as isize,
                pivots_serial[(rankn - 1) * (i + 1)]
            );
        }
    }
    comm::barrier();

    comm::broadcast(&mut loc_pivots[rank], 0);
    comm::barrier();

    /* debug */ {
        let mut loc_pivots_serial = loc_pivots[rank].arget(rankn - 1);
        println!("rank {}: {:?}", rank, loc_pivots_serial); comm::barrier();
    }

    // partitioning by pivots
    let mut buckets: Vec<GlobalPointer<u32>> = Vec::new();
    buckets.resize(rankn, GlobalPointer::null());
    buckets[rank] = GlobalPointer::init(config, size + rankn);
    comm::barrier();

    let mut _j: usize = 0;
    let mut _k: usize = 1;
    let mut _i: usize = 0;
    while _i < n {
        if _j < rankn - 1 {
            if loc_data_serial[_i] < loc_pivots[rank].idx_rget(_j as isize) {
                buckets[rank].idx_rput(((n + 1) * _j + _k) as isize, loc_data_serial[_i]);
                _k += 1
            } else {
                buckets[rank].idx_rput(((n + 1) * _j) as isize, (_k - 1) as u32);
                _k = 1;
                _j += 1;
                _i -= 1;
            }
        } else {
            buckets[rank].idx_rput(((n + 1) * _j + _k) as isize, loc_data_serial[_i]);
            _k += 1;
        }
        _i += 1;
    }
    buckets[rank].idx_rput(((n + 1) * _j) as isize, (_k - 1) as u32);
    comm::barrier();

    /* debug */ {
        let mut loc_buckets = buckets[rank].arget(size + rankn);
        println!("rank {}: {:?}", rank, loc_buckets); comm::barrier();
    }
}