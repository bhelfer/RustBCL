#![allow(dead_code)]
mod shmemx;
mod global_pointer;

extern crate libc;
use self::libc::{c_int, size_t};
use std::slice;
use global_pointer::{Config, GlobalPointer};

#[link(name="oshmem", kind="dylib")]
extern {
    fn shmem_init();
    fn shmem_finalize();
    fn shmem_n_pes() -> libc::c_int;
    fn shmem_my_pe() -> libc::c_int;
    fn shmem_barrier_all();
    fn shmem_malloc(size: libc::size_t) -> *mut u8;
    fn shmem_free(ptr: *mut u8);
    fn shmem_putmem(target: *mut u8, source: *const u8, len: size_t, pe: c_int);
    fn shmem_getmem(target: *mut u8, source: *const u8, len: size_t, pe: c_int);
}

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

fn test_shmem() {
    // The statements here will be executed when the compiled binary is called
    // Print text to the console
    shmemx::init();
    let my_pe = shmemx::my_pe();
    let n_pes = shmemx::n_pes();

    unsafe {
        let source_ptr: *mut u8 = shmem_malloc(n_pes);
        let target_ptr = shmem_malloc(n_pes);
        let source_slice = slice::from_raw_parts_mut(source_ptr, n_pes);
        let target_slice = slice::from_raw_parts_mut(target_ptr, n_pes);

        for i in 0..n_pes {
            source_slice[i] = i as u8;
        }

        for i in 0..n_pes {
            target_slice[i] = 0 as u8;
        }

        if my_pe == 0 {
            shmem_putmem(target_ptr, source_ptr, n_pes / 2, 1);
        }
        if my_pe == 1 {
            shmem_getmem(target_ptr.add(n_pes / 2), source_ptr.add(n_pes / 2), n_pes / 2, 0);
        }
        shmemx::barrier();

        if my_pe == 1 {
            println!("Hello World! I am process {} out of {}",
             shmemx::my_pe(), shmemx::n_pes());
            for i in 0..n_pes {
                print!(" {}", target_slice[i]);
            }
            println!();
        }

        shmem_free(source_ptr);
        shmem_free(target_ptr);
    }
    shmemx::barrier();
    shmemx::finalize();
}