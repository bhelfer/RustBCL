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

#[derive(Debug, Clone)]
pub struct HashTable<K, V> {
    global_size: usize,
    local_size: usize,
    hash_table: Vec<GlobalPointer<HE<K, V>>>,
    used: Vec<GlobalPointer<i32>>,
}

impl<K, V> HashTable<K, V>
    where K: Clone + Hash + Copy + Default + Debug + PartialEq<K>,
          V: Clone + Copy + Default + Eq + PartialEq<V>,
{
    pub fn new(config: &mut Config, size: usize) -> Self {
        let local_size = (size + config.rankn - 1) / config.rankn;
        let global_size = local_size * config.rankn;

        // used record GlobalPointer
        let mut used: Vec<GlobalPointer<i32>> = Vec::new();
        used.resize(config.rankn, GlobalPointer::null());
        used[config.rank] = config.alloc::<i32>(local_size);
        for i in 0..local_size {
            unsafe {
                (used[config.rank] + i).local().write(0);
            }
        }
        for rank in 0..config.rankn {
            comm::broadcast(&mut used[rank], rank);
        }
        config.barrier();

        // hash entry GlobalPointer
        let mut hash_table: Vec<GlobalPointer<HE<K, V>>> = Vec::new();
        hash_table.resize(config.rankn, GlobalPointer::null());
        hash_table[config.rank] = config.alloc::<HE<K, V>>(local_size);
        for i in 0..local_size {
            unsafe {
                (hash_table[config.rank] + i).local().write(HashEntry::null());
            }
        }
        for rank in 0..config.rankn {
            comm::broadcast(&mut hash_table[rank], rank);
        }
        config.barrier();

        Self {
            global_size,
            local_size,
            hash_table,
            used,
        }
    }

    fn slot_entry_ptr(&self, slot: usize) -> GlobalPointer<HE<K, V>> {
        let node = slot / self.local_size;
        let node_slot = slot - node * self.local_size;

        if node >= shmemx::n_pes() { panic!("HashTable::slot_entry_ptr: node {} out of bound!", node); }
        if node_slot >= self.local_size { panic!("HashTable::slot_entry_ptr: node_slot {} out of bound!", node_slot); }

        (self.hash_table[node] + node_slot)
    }

    fn slot_used_ptr(&self, slot: usize) -> GlobalPointer<i32> {
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


    fn slot_status(&self, slot: usize) -> i32 {
        self.slot_used_ptr(slot).rget()
    }

    /* Request slot for key. If slot's free, take it.
       If slot's taken (ready_flag), reserve it (reserve_flag),
       so that you can write to it. */
    fn request_slot(&self, slot: usize) -> bool {

        let mut used_ptr: GlobalPointer<i32> = self.slot_used_ptr(slot);

//        println!("rank, node, node_slot, slot_status = {}, {}, {}, {}", shmemx::my_pe(), node,
//                 node_slot, self.slot_status(slot));
//        println!("used_ptr: {:?}", used_ptr);
//        println!("offset: 0x{:x}", used_ptr.offset * size_of::<i32>());


//        Why compare & swap lead to error ???
//        let origin = comm::int_compare_and_swap(&mut used_ptr, 0, 1);


        // use fetch & inc instead
        let origin = comm::int_finc(&mut used_ptr);
        if origin > 0 { false } else { true }
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

            println!("(cur, dst) = {}, {}, Requesting slot {}, key = {:?}", shmemx::my_pe(), slot / self.local_size, slot, key);

            success = self.request_slot(slot);

            if success {
                let mut entry: HE<K, V> = self.get_entry(slot);
                entry.set(&key, &value);
                self.set_entry(slot, &entry);

                if self.slot_status(slot) == 0 {
                    panic!("HashTable::insert: Bad slot({}) status", slot);
                }
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

        loop {
            let slot: usize = ((hash + probe) % (self.global_size as u64)) as usize;
            probe += 1;

            if self.slot_status(slot) > 0 {
                entry = self.get_entry(slot);
                success = (entry.get_key() == *key);
            }

            if success || probe >= self.global_size as u64 { break; }
        }

        if success {
            *value = entry.get_value();
            return true;
        } else {
            return false;
        }
    }
}