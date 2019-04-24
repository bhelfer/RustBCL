use shmemx;
use std::mem::size_of;
use shmemx::libc::{c_long, c_void, c_int};
use global_pointer::GlobalPointer;

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

pub fn int_compare_and_swap(ptr: &mut GlobalPointer<c_int>, old_val: c_int, new_val: c_int) -> c_int {

    let rank = ptr.rank;
    unsafe {
        shmemx::shmem_int_cswap(
            ptr.rptr() as *mut c_int,
            old_val as c_int,
            new_val as c_int,
            rank as c_int
        )
    }
}

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

pub fn int_finc(ptr: &mut GlobalPointer<i32>) -> i32 {
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

// added by lfz
pub fn quiet() {
    unsafe {
        shmemx::shmem_quiet();
    }
}

// added by lfz
pub fn set_lock(lock: *mut c_long) {
    unsafe {
        shmemx::shmem_set_lock(lock);
    }
}

// added by lfz
pub fn clear_lock(lock: *mut c_long) {
    unsafe {
        shmemx::shmem_clear_lock(lock);
    }
}

// added by lfz
pub fn test_lock(lock: *mut c_long) -> c_int {
    unsafe {
        shmemx::shmem_test_lock(lock)
    }
}
