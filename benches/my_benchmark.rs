#[macro_use]
extern crate criterion;

extern crate lib_bcl;
extern crate rand;

use criterion::Criterion;
use criterion::black_box;

use lib_bcl::hash_table;
use lib_bcl::config::Config;
use lib_bcl::global_pointer;
use lib_bcl::comm;
use lib_bcl::shmemx;
use std::collections::HashMap;

use hash_table::HashTable;
use self::rand::{Rng, SeedableRng, StdRng};
use global_pointer::GlobalPointer;

fn same_entry_test() -> i32 {

    let mut config = Config::init(32);
    let rankn: i64 = config.rankn as i64;
    let rank: i64 = config.rank as i64;

    let n: i64 = 10;
    let m: i64 = 10;

    let mut hash_table_ref: HashMap<i64, i64> = HashMap::new();
    let mut hash_table_lfz: HashTable<i64, i64> = HashTable::new(&mut config, (n*5) as usize);

    let mut k_ptr: GlobalPointer<i64> = GlobalPointer::null();
    let mut v_ptr: GlobalPointer<i64> = GlobalPointer::null();
    if rank == 0 {
        k_ptr = config.alloc::<i64>(1);
        v_ptr = config.alloc::<i64>(1);
    }
    comm::barrier();

    comm::broadcast(&mut k_ptr, 0);
    comm::broadcast(&mut v_ptr, 0);
    comm::barrier();

    let mut rng: StdRng = SeedableRng::from_seed([233; 32]);

    for i in 0 .. n {
        if rank == 0 {
            k_ptr.rput(rng.gen_range(-m, m));
            v_ptr.rput(rng.gen_range(-m, m));
        }
        comm::barrier();

        let key = k_ptr.rget();
        let value = v_ptr.rget();
        comm::barrier();

        // all PE
        let success = hash_table_lfz.insert(&key, &value);
        hash_table_ref.insert(key.clone(), value.clone());

        if success == false {
            panic!("HashTable({}) Agh! insertion failed", shmemx::my_pe());
        }

        comm::barrier();
    }

    comm::barrier();
    println!("HashTable({}) Done with insert!", shmemx::my_pe());
    comm::barrier();

    comm::barrier();

    for i in -m .. m {
        if (rank - i) % rankn == 0 {
            let v_ref = hash_table_ref.get(&i);
            let v_ref = match v_ref {
                Some(&v) => v,
                None => std::i64::MAX,
            };

            let mut v_lfz: i64 = 0;
            let mut success: bool = false;
            success = hash_table_lfz.find(&i, &mut v_lfz);

            if !success {
                v_lfz = std::i64::MAX;
            }

            println!("iter_find({}) {}, (v_ref, v_lfz) = ({}, {})", rank, i, v_ref, v_lfz);
            assert_eq!(v_ref, v_lfz);
        }

        comm::barrier();
    }

    return 0;
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("same_entry test", |b| b.iter(|| same_entry_test()));
//    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);