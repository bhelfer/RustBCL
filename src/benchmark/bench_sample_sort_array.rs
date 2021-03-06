#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]
#![allow(non_snake_case)]

extern crate is_sorted;
extern crate quickersort;

use base::{global_pointer::GlobalPointer, config::Config};
use backend::comm;

use std::time::{SystemTime, Duration};
use std::vec::Vec;
use std::env;
use rand::{rngs::StdRng, Rng, thread_rng, SeedableRng, ChaChaRng};
use is_sorted::IsSorted;
use containers::array::Array;

pub fn benchmark_sample_sort(config: &mut Config) {

    let args: Vec<String> = env::args().collect();
    let mut n: usize;
    let min_size: usize = 8;
    // output debug info or not
    let mut DBG: bool = true;
    let mut DETAIL: bool = true;

    let rankn: usize = config.rankn as usize;
    let rank: usize = config.rank as usize;

    if args.len() <= 1 {
        n = min_size;
        println!("not enough arguments\nUse default argument n = {}", n);
    } else {
        n = args[1].clone().parse().unwrap();
        if n < min_size { n = min_size; }
    }

    if n * rankn >= 100 { DBG = false; }

    let mut rng: StdRng = SeedableRng::from_seed([rankn as u8; 32]);

    let size: usize = n * rankn;

    let mut input: GlobalPointer<u32> = GlobalPointer::init(config, size);
    if rank == 0 {
        for i in 0 .. size {
            input.idx_rput(
                i as isize,
                rng.gen_range(0, std::u32::MAX)
            );
        }
    }
    comm::barrier();

    /* debug */ if DBG { println!("input = {:?}", input); comm::barrier(); }

    // here start the sorting
    comm::barrier();
    let start_time = SystemTime::now();

    // 0. scattering data from rank 0
    let start_time_0 = SystemTime::now();

    let mut loc_data: Array<u32> = Array::init(config, size);
    comm::barrier();

    comm::scatter(&mut loc_data.ptrs[rank], &mut input, 0, n);
    comm::barrier();

    let total_time_0 = SystemTime::now().duration_since(start_time_0)
        .expect("SystemTime::duration_since failed");
    if rank == 0 && DETAIL { println!("scattering data in {:?}", total_time_0); }


    // 1. local sorting(user serial type)
    let start_time_1 = SystemTime::now();

    let mut loc_data_serial = loc_data.ptrs[rank].arget(n);
    comm::barrier();
    quickersort::sort(&mut loc_data_serial[..]);
    comm::barrier();

    /* debug */ if DBG {
        assert_eq!(IsSorted::is_sorted(&mut loc_data_serial.iter()), true);
        println!("rank {}: {:?}", rank, loc_data_serial); comm::barrier();
    }

    comm::barrier();
    let total_time_1 = SystemTime::now().duration_since(start_time_1)
        .expect("SystemTime::duration_since failed");
    if rank == 0 && DETAIL { println!("local sorting in {:?}", total_time_1); }


    // 2. choosing local pivots
    let start_time_2 = SystemTime::now();

    let mut loc_pivots: Array<u32> = Array::init(config, rankn * (rankn - 1));
    comm::barrier();
    for i in 0 .. rankn - 1 {
        loc_pivots.ptrs[rank].idx_rput(
            i as isize,
            loc_data_serial[size / (rankn * rankn) * (i + 1)]
        );
    }

    comm::barrier();
    let total_time_2 = SystemTime::now().duration_since(start_time_2)
        .expect("SystemTime::duration_since failed");
    if rank == 0 && DETAIL { println!("choosing local pivots in {:?}", total_time_2); }

    // 3. gather local pivots to rank 0
    let start_time_3 = SystemTime::now();

    let mut pivots: GlobalPointer<u32> = GlobalPointer::init(config, rankn * (rankn - 1));
    comm::gather(&mut pivots, &mut loc_pivots.ptrs[rank], 0, rankn - 1);
    comm::barrier();
    let total_time_3 = SystemTime::now().duration_since(start_time_3)
        .expect("SystemTime::duration_since failed");
    if rank == 0 && DETAIL { println!("gather local pivots to rank 0 in {:?}", total_time_3); }

    // 4. sort all pivots in rank 0
    let start_time_4 = SystemTime::now();

    if rank == 0 {
        let mut pivots_serial = pivots.arget(rankn * (rankn - 1));
        quickersort::sort(&mut pivots_serial[..]);

        /* debug */ if DBG {  println!("root is rank {}: {:?}", rank, pivots_serial); }
        for i in 0 .. rankn - 1 {
            loc_pivots.get_ptr(0).idx_rput(
                i as isize,
                pivots_serial[(rankn - 1) * (i + 1)]
            );
        }
    }
    comm::barrier();

    comm::broadcast(&mut loc_pivots.ptrs[rank], 0);
    comm::barrier();
    let total_time_4 = SystemTime::now().duration_since(start_time_4)
        .expect("SystemTime::duration_since failed");
    if rank == 0 && DETAIL { println!("sort all pivots in rank 0 in {:?}", total_time_4); }

    /* debug */ if DBG {
        let mut loc_pivots_serial = loc_pivots.ptrs[rank].arget(rankn - 1);
        println!("rank {}: {:?}", rank, loc_pivots_serial); comm::barrier();
    }


    // 5. partitioning by pivots (spare many empty spaces)
    // (notice: this is not the optimal algorithm, but easy to all_to_all broadcast)

    let start_time_5 = SystemTime::now();

    let mut buckets: Array<u32> = Array::init(config, rankn * (size + rankn));
    comm::barrier();

    let mut i: usize = 0;
    let mut j: usize = 0;
    let mut k: usize = 1;
    while i < n {
        if j < rankn - 1 {
            if loc_data_serial[i] < loc_pivots.ptrs[rank].idx_rget(j as isize) {
                buckets.ptrs[rank].idx_rput(((n + 1) * j + k) as isize, loc_data_serial[i]);
                k += 1
            } else {
                buckets.ptrs[rank].idx_rput(((n + 1) * j) as isize, (k - 1) as u32);
                k = 1;
                j += 1;
                i -= 1;
            }
        } else {
            buckets.ptrs[rank].idx_rput(((n + 1) * j + k) as isize, loc_data_serial[i]);
            k += 1;
        }
        i += 1;
    }
    buckets.ptrs[rank].idx_rput(((n + 1) * j) as isize, (k - 1) as u32);
    comm::barrier();

    let total_time_5 = SystemTime::now().duration_since(start_time_5)
        .expect("SystemTime::duration_since failed");
    if rank == 0 && DETAIL { println!("partitioning by pivots (spare many empty spaces) in {:?}", total_time_5); }

    /* debug */ if DBG {
        let mut buckets_serial = buckets.ptrs[rank].arget(size + rankn);
        println!("rank {}: buckets = {:?}", rank, buckets_serial); comm::barrier();
    }


    // 6. exchange local buckets
    let start_time_6 = SystemTime::now();

    let mut swap_buckets: Array<u32> = Array::init(config, rankn * (size + rankn));
    comm::barrier();

    comm::all_to_all(&mut swap_buckets.ptrs[rank], &mut buckets.ptrs[rank], n + 1);
    comm::barrier();

    let total_time_6 = SystemTime::now().duration_since(start_time_6)
        .expect("SystemTime::duration_since failed");
    if rank == 0 && DETAIL { println!("exchange local buckets in {:?}", total_time_6); }

    /* debug */ if DBG {
        let mut swap_buckets_serial = swap_buckets.ptrs[rank].arget(size + rankn);
        println!("rank {}: swap_buckets = {:?}", rank, swap_buckets_serial); comm::barrier();
    }


    // 7. rearrange buffers
    let start_time_7 = SystemTime::now();

    // maximum size is proofed on class
    let mut loc_buckets: Array<u32> = Array::init(config, rankn * (2 * size / rankn));
    comm::barrier();

    let mut pos: usize = 1;
    for j in 0 .. rankn {
        k = 1;
        let cnt = swap_buckets.ptrs[rank].idx_rget(((size / rankn + 1) * j) as isize);
        for i in 0 .. cnt {
            loc_buckets.ptrs[rank].idx_rput(
                pos as isize,
                swap_buckets.ptrs[rank].idx_rget(((size / rankn + 1) * j + k) as isize)
            );
            k += 1;
            pos += 1;
        }
    }
    loc_buckets.ptrs[rank].idx_rput(0, (pos - 1) as u32);
    comm::barrier();

    let total_time_7 = SystemTime::now().duration_since(start_time_7)
        .expect("SystemTime::duration_since failed");
    if rank == 0 && DETAIL { println!("rearrange buffers in {:?}", total_time_7); }

    // 8. local sorting
    let start_time_8 = SystemTime::now();

    let mut loc_buckets_serial = (loc_buckets.ptrs[rank] + 1).arget(pos - 1);
    quickersort::sort(&mut loc_buckets_serial[..]);

    /* debug */ if DBG {
        println!("rank {}: loc_buckets = {:?}", rank, loc_buckets_serial); comm::barrier();
    }

    (loc_buckets.ptrs[rank] + 1).arput(&loc_buckets_serial);
    comm::barrier();
    let total_time_8 = SystemTime::now().duration_since(start_time_8)
        .expect("SystemTime::duration_since failed");
    if rank == 0 && DETAIL { println!("rearranged buffers local sorting in {:?}", total_time_8); }

    // 9. gathering sorted buckets to rank 0
    let start_time_9 = SystemTime::now();

    let mut out_bucket: GlobalPointer<u32> = GlobalPointer::init(config, 2 * size);
    comm::gather(&mut out_bucket, &mut loc_buckets.ptrs[rank], 0, 2 * n);
    comm::barrier();
    let total_time_9 = SystemTime::now().duration_since(start_time_9)
        .expect("SystemTime::duration_since failed");
    if rank == 0 && DETAIL { println!("gathering sorted buckets to rank 0 in {:?}", total_time_9); }

    /* debug */ if DBG {
        if rank == 0 {
            let mut out_bucket_serial = out_bucket.arget(size);
            println!("rank {}: out_bucket = {:?}", rank, out_bucket_serial);
        }
        comm::barrier();
    }

    // 10. rearraging buffers
    let start_time_10 = SystemTime::now();

    let mut output: Vec<u32> = Vec::new();
    if rank == 0 {
        output.resize(size, 0);

        pos = 0;
        for j in 0 .. rankn {
            k = 1;
            let cnt = out_bucket.idx_rget(((2 * size / rankn) * j) as isize);
            for i in 0 .. cnt {
                output[pos] = out_bucket.idx_rget(((2 * size / rankn) * j + k) as isize);
                k += 1;
                pos += 1;
            }
        }
        output.resize(pos, 0);
    }
    let total_time_10 = SystemTime::now().duration_since(start_time_10)
        .expect("SystemTime::duration_since failed");
    if rank == 0 && DETAIL { println!("rearraging buffers in {:?}", total_time_10); }

    comm::barrier();
    let total_time = SystemTime::now().duration_since(start_time)
        .expect("SystemTime::duration_since failed");


    if rank == 0 {
        let mut input = input.arget(size);
        quickersort::sort(&mut input[..]);

//        println!("out.len(), in.len() = {}, {}", output.len(), input.len());
        assert_eq!(output, input);

        /* do not want to crash the screen */
        if DBG {
            /* debug */ { println!(); }
            /* debug */ { println!("rank {}: output = {:?}", rank, output); }
            /* debug */ { println!("rank {}: input = {:?}", rank, input); }
        }

        println!("total_time = {:?}", total_time);
    }

    comm::barrier();
}
