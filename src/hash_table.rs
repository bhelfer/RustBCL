#![allow(dead_code)]
#![allow(unused)]

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

#[derive(Debug, Copy, Clone)]
struct HashEntry<K, V> {
    key: K,
    value: V,
}

impl<K, V> HashEntry<K, V>
    where K: Clone + Hash + Copy + Default + Debug + PartialEq<K>,
          V: Clone + Copy + Default + PartialEq<V>,
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
          V: Clone + Copy + Default + Eq + PartialEq<V>,
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
        for i in 0..local_size {
            (used[config.rank] + i).rput(free_flag);
//            unsafe {
//                (used[config.rank] + i).local().write(free_flag);
//            }
        }
        for rank in 0..config.rankn {
            comm::broadcast(&mut used[rank], rank);
        }
        Config::barrier();

        // hash entry GlobalPointer
        let mut hash_table: Vec<GlobalPointer<HE<K, V>>> = Vec::new();
        hash_table.resize(config.rankn, GlobalPointer::null());
        hash_table[config.rank] = config.alloc::<HE<K, V>>(local_size);
        for i in 0..local_size {
            (hash_table[config.rank] + i).rput(HashEntry::null());
//            unsafe {
//                (hash_table[config.rank] + i).local().write(HashEntry::null());
//            }
        }
        for rank in 0..config.rankn {
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

        (self.hash_table[node] + node_slot)
    }

    fn slot_used_ptr(&self, slot: usize) -> GlobalPointer<U> {
        let node = slot / self.local_size;
        let node_slot = slot - node * self.local_size;

        if node >= shmemx::n_pes() { panic!("HashTable::slot_used_ptr: node {} out of bound!", node); }
        if node_slot >= self.local_size { panic!("HashTable::slot_used_ptr: node_slot {} out of bound!", node_slot); }

        (self.used[node] + node_slot)
    }

    fn get_entry(&self, slot: usize) -> HE<K, V> {
        let node = slot / self.local_size;
        let node_slot = slot - node * self.local_size;

        if node >= shmemx::n_pes() { panic!("HashTable::get_entry: node {} out of bound!", node); }
        if node_slot >= self.local_size { panic!("HashTable::get_entry: node_slot {} out of bound!", node_slot); }

        self.slot_entry_ptr(slot).rget()
    }

    fn set_entry(&self, slot: usize, entry: &HE<K, V>) {
        let node = slot / self.local_size;
        let node_slot = slot - node * self.local_size;

        if node >= shmemx::n_pes() { panic!("HashTable::set_entry: node {} out of bound!", node); }
        if node_slot >= self.local_size { panic!("HashTable::set_entry: node_slot {} out of bound!", node_slot); }

        self.slot_entry_ptr(slot).rput(*entry);
    }

    fn slot_status(&self, slot: usize) -> U {
        self.slot_used_ptr(slot).rget()
    }

    fn make_ready_slot(&self, slot: usize) {
        let mut used_ptr: GlobalPointer<U> = self.slot_used_ptr(slot);
        let used_val = comm::long_compare_and_swap(
            &mut used_ptr,
            self.reserved_flag,
            self.ready_flag
        );
        // TODO: if we fix updates to atomic, cannot be ready_flag
        if !(used_val == self.reserved_flag || used_val == self.ready_flag) {
            panic!("HashMap forqs: used flag was somehow corrupted (-> ready_flag). \
                    got {} at node {}", used_val, slot / self.local_size);
        }
    }

    /* Request slot for key. If slot's free, take it.
       If slot's taken (ready_flag), reserve it (reserve_flag),
       so that you can write to it. */
    fn request_slot(&self, slot: usize, key: &K) -> bool {

        let mut used_ptr: GlobalPointer<U> = self.slot_used_ptr(slot);
        let mut used_val: c_long = self.free_flag;
        /* If someone is currently inserting into this slot (reserved_flag), wait
         until they're finished to proceed. */
        loop {
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
            panic!("HashMap forqs: used flag was somehow corrupted (-> reserved_flag). \
                    got {} at node {}", used_val, slot / self.local_size);
        }

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
                return true;
            } else {
                // not to update, request fail
                return false;
            }
        } else {
            // slot free
            return true;
        }
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

            println!("HashTable::insert: (cur, dst) = {}, {}, Requesting slot {}, key = {:?}", shmemx::my_pe(), slot / self.local_size, slot, key);

            success = self.request_slot(slot, &key);

            if success {
                let mut entry: HE<K, V> = self.get_entry(slot);
                entry.set(&key, &value);
                self.set_entry(slot, &entry);
                self.make_ready_slot(slot);
            }

            if success || probe >= self.global_size as u64 { break; }
        }

        success
    }

    pub fn find(&self, key: &K, value: &mut V) -> bool {
        let hash = self.get_hash(&key);

        let mut probe: u64 = 0;
        let mut success = false;

        let mut entry: HashEntry<K, V> = HashEntry::null();
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