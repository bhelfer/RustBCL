#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

use base::global_pointer::GlobalPointer;
use backend::shmemx::{self, libc::{c_long, c_void, c_int}};
use std::mem::size_of;

pub fn broadcast<T>(val: &mut T, root: usize) {
    unsafe{
        let p_sync_ptr: *mut c_long = shmemx::shmem_malloc(shmemx::_SHMEM_BCAST_SYNC_SIZE * size_of::<c_long>()) as *mut c_long;
        if p_sync_ptr.is_null() {
            panic!("BCL: Could not allocate shared memory segment.")
        }
        let p_sync_slice = std::slice::from_raw_parts_mut(p_sync_ptr, shmemx::_SHMEM_BCAST_SYNC_SIZE);
        for i in 0..shmemx::_SHMEM_BCAST_SYNC_SIZE {
            p_sync_slice[i] = shmemx::_SHMEM_SYNC_VALUE;
        }
//        shmemx::barrier();

        let nelem = (size_of::<T>() + 7) / 8;
//        println!("broadcast: size<t>:{}, nelem={}", size_of::<T>(), nelem);
        let rv = shmemx::shmem_malloc(nelem * 64) as *mut u64;
        if rv.is_null() {
            panic!("BCL: Could not allocate shared memory segment.")
        }
        let startval = shmemx::shmem_malloc(nelem * 64) as *mut u64;
        if startval.is_null() {
            panic!("BCL: Could not allocate shared memory segment.")
        }

        if shmemx::my_pe() == root {
            libc::memcpy(startval as *mut c_void, val as *const T as *const c_void, size_of::<T>());
        }
        shmemx::shmem_broadcast64(rv, startval, nelem, root as i32, 0, 0, shmemx::n_pes() as i32, p_sync_ptr);
        if shmemx::my_pe() != root {
            libc::memcpy(val as *mut T as *mut c_void, rv as *const c_void, size_of::<T>());
        }
        shmemx::shmem_free(p_sync_ptr as *mut u8);
        shmemx::shmem_free(rv as *mut u8);
        shmemx::shmem_free(startval as *mut u8);
    }
}

pub fn barrier() {
    shmemx::barrier();
}

// lock
pub type LockT = i64;
pub const LOCK_SIZE: usize = 8;
pub const UNLOCKED: LockT = 1111;
pub const LOCKED: LockT = 2222;
unsafe fn atomic_compare_swap(dest: *mut LockT, cond: LockT, value: LockT, pe: i32) -> LockT {
	shmemx::shmem_long_atomic_compare_swap(dest, cond, value, pe)
}

pub fn set_lock(lock: *mut LockT, rank: usize) {
    loop {
        unsafe {
            let current = shmemx::shmem_long_atomic_fetch(lock, rank as c_int);
            if current != LOCKED && current != UNLOCKED {
                panic!("not a lock!");
            }
            if current == UNLOCKED {
                let expected = UNLOCKED;
                let status = atomic_compare_swap(lock, UNLOCKED, LOCKED, rank as i32);
                if status == expected {
                    break;
                }
            }
            // emit a pause
        }
    }
}

pub fn clear_lock(lock: *mut LockT, rank: usize) {
    unsafe {
        shmemx::shmem_long_atomic_set(lock, UNLOCKED, rank as i32);
    }
}

pub fn test_lock(lock: *mut LockT, rank: usize) -> bool {
    unsafe {
        let current = shmemx::shmem_long_atomic_fetch(lock, rank as c_int);
        if current != LOCKED && current != UNLOCKED {
            panic!("not a lock!");
        }
        if current == UNLOCKED {
            let expected = UNLOCKED;
            let status = shmemx::shmem_long_atomic_compare_swap(lock, UNLOCKED, LOCKED, rank as i32);
            if status == expected {
                return true;
            }
        }
        return false;
    }
}

// some functions
pub fn long_compare_and_swap(ptr: &mut GlobalPointer<c_long>, old_val: c_long, new_val: c_long) -> c_long {

    let rank = ptr.rank;
    unsafe {
        shmemx::shmem_long_atomic_compare_swap(
            ptr.rptr() as *mut c_long,
            old_val as c_long,
            new_val as c_long,
            rank as c_int
        )
    }
}

pub fn int_atomic_fetch_inc(ptr: &mut GlobalPointer<i32>) -> i32 {
    let rank = ptr.rank;
    unsafe {
        shmemx::shmem_int_atomic_fetch_inc(
            ptr.rptr() as *mut i32,
            rank as c_int
        )
    }
}

pub fn long_atomic_fetch(ptr: &mut GlobalPointer<c_long>) -> c_long {
    unsafe {
        shmemx::shmem_long_atomic_fetch(
            ptr.rptr() as *const c_long,
            ptr.rank as c_int
        )
    }
}

pub fn int_atomic_fetch(ptr: &mut GlobalPointer<c_int>) -> c_int {
    unsafe {
        shmemx::shmem_int_atomic_fetch(
            ptr.rptr() as *const c_int,
            ptr.rank as c_int
        )
    }
}

pub fn int_atomic_fetch_and(ptr: &mut GlobalPointer<c_int>, value: c_int) -> c_int {
    unsafe {
        shmemx::shmem_int_atomic_fetch_and(
            ptr.rptr() as * mut c_int,
            value, ptr.rank as c_int
        )
    }
}

pub fn long_atomic_fetch_and(ptr: &mut GlobalPointer<c_long>, value: c_long) -> c_long {
    unsafe {
        shmemx::shmem_long_atomic_fetch_and(
            ptr.rptr() as * mut c_long,
            value, ptr.rank as c_long
        )
    }
}
pub fn long_atomic_fetch_add(ptr: &mut GlobalPointer<c_long>, value: c_long) -> c_long {
    unsafe {
        shmemx::shmem_long_atomic_fetch_add(
            ptr.rptr() as * mut c_long,
            value, ptr.rank as c_long
        )
    }
}

pub fn int_atomic_fetch_add(ptr: &mut GlobalPointer<c_int>, value: c_int) -> c_int {
    unsafe {
        shmemx::shmem_int_atomic_fetch_add(
            ptr.rptr() as *mut c_int,
            value,
            ptr.rank as c_int
        )
    }
}

pub fn long_atomic_fetch_xor(ptr: &mut GlobalPointer<c_long>, value: c_long) -> c_long {
    unsafe {
        shmemx::shmem_long_atomic_fetch_xor(
            ptr.rptr() as * mut c_long,
            value, ptr.rank as c_long
        )
    }
}

// added by lfz
pub fn fence() {
    unsafe {
        shmemx::shmem_fence();
    }
}

