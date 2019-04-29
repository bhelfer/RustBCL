use global_guard::GlobalGuard;
use config::Config;

pub fn benchmark_global_guard(config: &mut Config) {
    // ----------- Global Guard's part -------------
    if config.rank == 0 { println!("------------Global Guard's benchmark------------\n"); }

    let iter = 100; // iter per step
    let step = 1000;
    let mut guard1 = GlobalGuard::null();
    if config.rank == 0 {
        guard1 = GlobalGuard::init(config);
    }
    comm::broadcast(&mut guard1, 0);
    // println!("rank:{}, guard1:{:?}", config.rank, guard1);

    if config.rank == 0 {
        let value = guard1.lock();
        value.rput(0);
    }
    comm::barrier();

    // text mutex
    if config.rank == 1 {
        for _ in 0..step {
            for _ in 0..iter {

            }
        }
    }
    for i in 0..step {
        let value = guard1.lock();
        let t = value.rget();
        value.rput(t + 1);
    }
    comm::barrier();

    if config.rank == 0 {
        let value = guard1.lock();
        let t = value.rget();
        assert_eq!(t, step * config.rankn);
        println!("Global Guard's test: pass! step: {}", step);
    }
}