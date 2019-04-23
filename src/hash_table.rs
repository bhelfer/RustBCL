#![allow(dead_code)]
#![allow(unused)]
#![allow(deprecated)]

use global_pointer;
use comm;
use config;
use config::Config;
use shmemx;
use std::marker::PhantomData;
use global_pointer::GlobalPointer;
use shmemx::shmem_broadcast64;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::ptr::null;
use std::mem::size_of;
use shmemx::libc::{c_long, c_void, c_int};
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::{stdout, Write};
use std::thread::sleep;
use std::thread::sleep_ms;

#[derive(Debug, Copy, Clone)]
struct HashEntry<K, V> {
    key: K,
    value: V,
}

impl<K, V> HashEntry<K, V>
    where K: Clone + Hash + Copy + Default + Debug + PartialEq<K>,
          V: Clone + Copy + Default + Debug + PartialEq<V>,
{
    pub fn new(key: K, value: V) -> Self {
        Self { key, value }
    }

    pub fn null() -> Self {
        Self { key: Default::default(), value: Default::default() }
    }

    pub fn set(&mut self, key: &K, value: &V) {
        self.key = key.clone();
        self.value = value.clone();
    }

    pub fn get_key(&self) -> K {
        self.key
    }
    pub fn get_value(&self) -> V {
        self.value
    }

}

type HT<K, V> = HashTable<K, V>;
type HE<K, V> = HashEntry<K, V>;
type U = c_long;

#[derive(Debug, Clone)]
pub struct HashTable<K, V> {
    global_size: usize,
    local_size: usize,
    hash_table: Vec<GlobalPointer<HE<K, V>>>,
    used: Vec<GlobalPointer<U>>,
    free_flag: U, // empty slot
    reserved_flag: U, // writing slot
    ready_flag: U, // finish slot
}

impl<K, V> HashTable<K, V>
    where K: Clone + Hash + Copy + Default + Debug + PartialEq<K>,
          V: Clone + Copy + Default + Debug + Eq + PartialEq<V>,
{
    pub fn new(config: &mut Config, size: usize) -> Self {
        let local_size = (size + config.rankn - 1) / config.rankn;
        let global_size = local_size * config.rankn;

        let free_flag: U = 0;
        let reserved_flag: U = 1;
        let ready_flag: U = 2;

        // used record GlobalPointer
        let mut used: Vec<GlobalPointer<U>> = Vec::new();
        used.resize(config.rankn, GlobalPointer::null());
        used[config.rank] = config.alloc::<U>(local_size);
        for i in 0 .. local_size {
            (used[config.rank] + i as isize).rput(free_flag);
//            unsafe {
//                (used[config.rank] + i).local().write(free_flag);
//            }
        }
        Config::barrier();
        for rank in 0 .. config.rankn {
            comm::broadcast(&mut used[rank], rank);
        }
        Config::barrier();

        // hash entry GlobalPointer
        let mut hash_table: Vec<GlobalPointer<HE<K, V>>> = Vec::new();
        hash_table.resize(config.rankn, GlobalPointer::null());
        hash_table[config.rank] = config.alloc::<HE<K, V>>(local_size);
        for i in 0 .. local_size {
            (hash_table[config.rank] + i as isize).rput(HashEntry::null());
//            unsafe {
//                (hash_table[config.rank] + i).local().write(HashEntry::null());
//            }
        }
        Config::barrier();
        for rank in 0 .. config.rankn {
            comm::broadcast(&mut hash_table[rank], rank);
        }
        Config::barrier();

        Self {
            global_size,
            local_size,
            hash_table,
            used,
            free_flag,
            reserved_flag,
            ready_flag,
        }
    }

    fn slot_entry_ptr(&self, slot: usize) -> GlobalPointer<HE<K, V>> {
        let node = slot / self.local_size;
        let node_slot = slot - node * self.local_size;

        if node >= shmemx::n_pes() { panic!("HashTable::slot_entry_ptr: node {} out of bound!", node); }
        if node_slot >= self.local_size { panic!("HashTable::slot_entry_ptr: node_slot {} out of bound!", node_slot); }

        (self.hash_table[node] + node_slot as isize)
    }

    fn slot_used_ptr(&self, slot: usize) -> GlobalPointer<U> {
        let node = slot / self.local_size;
        let node_slot = slot - node * self.local_size;

        if node >= shmemx::n_pes() { panic!("HashTable::slot_used_ptr: node {} out of bound!", node); }
        if node_slot >= self.local_size { panic!("HashTable::slot_used_ptr: node_slot {} out of bound!", node_slot); }

        (self.used[node] + node_slot as isize)
    }

    fn get_entry(&self, slot: usize) -> HE<K, V> {
        println!("HashTable({})::get_entry slot {} enter", shmemx::my_pe(), slot);
        let mut entry_ptr = self.slot_entry_ptr(slot);
        println!("HashTable({})::get_entry slot {} middle", shmemx::my_pe(), slot);
        let ret = entry_ptr.rget();
        println!("HashTable({})::get_entry slot {} leave", shmemx::my_pe(), slot);
        ret
    }

    fn set_entry(&self, slot: usize, entry: &HE<K, V>) {
        self.slot_entry_ptr(slot).rput(*entry);
    }

    fn slot_status(&self, slot: usize) -> U {
        self.slot_used_ptr(slot).rget()
//        comm::long_atomic_fetch(&mut self.slot_used_ptr(slot))
    }

    fn make_ready_slot(&self, slot: usize, key: &K, value: &V) {
        let mut used_ptr: GlobalPointer<U> = self.slot_used_ptr(slot);
        println!("HashTable({})::make_ready_slot (k, v) = ({:?}, {:?}) pos 2", shmemx::my_pe(), key, value);

        let used_val: U = comm::long_compare_and_swap(
            &mut used_ptr,
            self.reserved_flag,
            self.ready_flag
        );

        println!("HashTable({})::make_ready_slot (k, v) = ({:?}, {:?}) pos 3", shmemx::my_pe(), key, value);

        // TODO: if we fix updates to atomic, cannot be ready_flag
        if !(used_val == self.reserved_flag || used_val == self.ready_flag) {
            panic!("HashTable forqs: used flag was somehow corrupted (-> ready_flag). \
                    got {} at node {}", used_val, slot / self.local_size);
        }
    }

    /* Request slot for key. If slot's free, take it.
       If slot's taken (ready_flag), reserve it (reserve_flag),
       so that you can write to it. */
    fn request_slot(&self, slot: usize, key: &K, value: &V) -> bool {

        let mut used_ptr: GlobalPointer<U> = self.slot_used_ptr(slot);
        let mut used_val: U = self.free_flag;
        /* If someone is currently inserting into this slot (reserved_flag), wait
         until they're finished to proceed. */
        loop {
            if SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() % 500009 == 0 {
                println!("HashTable({})::request_slot (k, v) = ({:?}, {:?}) in loop 1", shmemx::my_pe(), key, value);
            }
            // TODO: possibly optimize subsequent CASs to rget's
            used_val = comm::long_compare_and_swap(
                &mut used_ptr,
                self.free_flag,
                self.reserved_flag
            );
            if used_val != self.reserved_flag { break; }
        }
        /* used_val is ready_flag (*used_ptr is ready_flag) or
         free_flag (*used_ptr is now reserved_flag) */
        if !(used_val == self.free_flag || used_val == self.ready_flag) {
            panic!("HashTable forqs: used flag was somehow corrupted (-> reserved_flag). \
                    got {} at node {}", used_val, slot / self.local_size);
        }

        let mut return_flag = true;
        if used_val == self.ready_flag {
            // slot inserted
            if self.get_entry(slot).get_key() == *key {
                // if to update inserted HashEntry<K, V>
                loop {
                    used_val = comm::long_compare_and_swap(
                        &mut used_ptr,
                        self.ready_flag,
                        self.reserved_flag
                    );
                    if used_val == self.ready_flag { break; }
                }
            } else {
                // not to update, request fail
                return_flag = false;
            }
        } else {
            // slot free
        }

        println!("HashTable({})::request_slot (k, v) = ({:?}, {:?}) leave with {}", shmemx::my_pe(), key, value, return_flag);
        return_flag
    }

    fn get_hash(&self, key: &K) -> u64 {
        let mut hasher =  DefaultHasher::new();
        Hash::hash(&key, &mut hasher);
        let hash = hasher.finish();

        hash
    }

    pub fn insert(&self, key: &K, value: &V) -> bool {
        let hash = self.get_hash(&key);

        let mut probe: u64 = 0;
        let mut success = false;

        loop {
            let slot: usize = ((hash + probe) % (self.global_size as u64)) as usize;
            probe += 1;

            println!("HashTable({})::insert (k, v) = ({:?}, {:?}) Requesting slot {}", shmemx::my_pe(), key, value, slot);

            success = self.request_slot(slot, &key, &value);
            assert_eq!(self.slot_status(slot), self.reserved_flag);

            if success {

                println!("HashTable({})::insert (k, v) = ({:?}, {:?}) Setting slot {} pos 1", shmemx::my_pe(), key, value, slot);

                let mut entry: HE<K, V> = self.get_entry(slot);

                println!("HashTable({})::insert (k, v) = ({:?}, {:?}) Setting slot {} pos 2", shmemx::my_pe(), key, value, slot);
                entry.set(&key, &value);

                self.set_entry(slot, &entry);

                self.make_ready_slot(slot, &key, &value);
                assert_ne!(self.slot_status(slot), self.free_flag);

            }

            if success || probe >= self.global_size as u64 { break; }
        }

        println!("HashTable({})::insert (k, v) = ({:?}, {:?}) leave with {}", shmemx::my_pe(), key, value, success);
        success
    }

    pub fn find(&self, key: &K, value: &mut V) -> bool {
        let hash = self.get_hash(&key);

        let mut probe: u64 = 0;
        let mut success = false;

        let mut entry: HE<K, V> = HashEntry::null();
        let mut status: U;

        loop {
            let slot: usize = ((hash + probe) % (self.global_size as u64)) as usize;

            probe += 1;
            status = self.slot_status(slot);

            if status == self.ready_flag {
                entry = self.get_entry(slot);
                success = (entry.get_key() == *key);
            }

            if success || status == self.free_flag || probe >= self.global_size as u64 { break; }
        }

        if success {
            *value = entry.get_value();
            return true;
        } else {
            return false;
        }
    }
}

//        let origin: i32 = comm::long_compare_and_swap(&mut used_ptr, 0, 1);
//        let origin = comm::int_finc(&mut used_ptr);
//        let origin = comm::int_compare_and_swap(&mut used_ptr, 0, 1);


#[cfg(test)]
pub mod tests {

    extern crate rand;
    use std::collections::HashMap;
    use hash_table::HashTable;
    use config::Config;
    use self::rand::{Rng, SeedableRng, StdRng};
    use global_pointer::GlobalPointer;
    use comm;
    use std::os::raw::c_long;
    use std::ptr::null_mut;

    #[test]
    pub fn same_entry_test() {

        let mut config = Config::init(32);
        let rankn: i64 = config.rankn as i64;
        let rank: i64 = config.rank as i64;

        let n: i64 = 100000;
        let m: i64 = 1000;

        let mut hash_table_ref: HashMap<i64, i64> = HashMap::new();
        let mut hash_table_lfz: HashTable<i64, i64> = HashTable::new(&mut config, (n*2) as usize);

        let mut k_ptr: GlobalPointer<i64> = GlobalPointer::null();
        let mut v_ptr: GlobalPointer<i64> = GlobalPointer::null();
        if rank == 0 {
            k_ptr = config.alloc::<i64>(1);
            v_ptr = config.alloc::<i64>(1);
        }
        Config::barrier();

        comm::broadcast(&mut k_ptr, 0);
        comm::broadcast(&mut v_ptr, 0);
        Config::barrier();

        let mut rng: StdRng = SeedableRng::from_seed([233; 32]);

        for i in 0 .. n {
            if rank == 0 {
                k_ptr.rput(rng.gen_range(-m, m));
                v_ptr.rput(rng.gen_range(-m, m));
            }
            Config::barrier();

            let key = k_ptr.rget();
            let value = v_ptr.rget();
            Config::barrier();

            // all PE
            hash_table_lfz.insert(&key, &value);
            hash_table_ref.insert(key.clone(), value.clone());

            Config::barrier();
        }

        Config::barrier();

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

                Config::barrier();

                if !success {
                    v_lfz = std::i64::MAX;
                }

                println!("iter_find({}) {}, (v_ref, v_lfz) = ({}, {})", rank, i, v_ref, v_lfz);
                assert_eq!(v_ref, v_lfz);
            }

            Config::barrier();
        }
    }
}


//    fn check_slot(&self, slot: usize) {
//        let used_val = self.slot_status(slot);
//        if used_val != self.ready_flag {
//            panic!("HashTable forqs: used flag was somehow corrupted (-> ready_flag). \
//                    got {} at node {}", used_val, slot / self.local_size);
//        }
//    }