use std::collections::HashMap;
use rustc_serialize::hex::ToHex;

pub trait Cache {
    fn exists(&self, height: u64, base: &Vec<u8>) -> bool;
    fn get(&self, height: u64, base: &Vec<u8>) -> Vec<u8>;
    fn hash_cache(&mut self, left: &Vec<u8>, right: &Vec<u8>, height: u64, base: &Vec<u8>, split: &Vec<u8>,
                  interior_hash: &Vec<u8>,
                  default_hashes: &Vec<Vec<u8>>);
    fn entries(&self) -> usize;
}

pub struct CacheNothing;

impl Cache for CacheNothing {
    fn exists(&self, height: u64, base: &Vec<u8>) -> bool {
        false
    }
    fn get(&self, height: u64, base: &Vec<u8>) -> Vec<u8> {
        Vec::new()
    }
    fn hash_cache(&mut self, left: &Vec<u8>, right: &Vec<u8>, height: u64, base: &Vec<u8>, split: &Vec<u8>,
                  interior_hash: &Vec<u8>,
                  default_hashes: &Vec<Vec<u8>>){
    }
    fn entries(&self) -> usize {
        0
    }
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Key{
    height: u64,
    base: Vec<u8>,
}

pub struct CacheBranch{
    map: HashMap<String, Vec<u8>>
}
impl CacheBranch{
    pub fn new()->CacheBranch{
        CacheBranch{map: HashMap::new()}
    }
}
impl Cache for CacheBranch{
    fn exists(&self, height: u64, base: &Vec<u8>) -> bool {
        let h_str = height.to_string();
        let key = format!("{}{}", h_str, base.to_hex());
        self.map.get(&key).is_some()
    }
    fn get(&self, height: u64, base: &Vec<u8>) -> Vec<u8> {
        let h_str = height.to_string();
        let key = format!("{}{}", h_str, base.to_hex());
        self.map.get(&key).expect("get should be used only after exists").clone()
    }
    fn hash_cache(&mut self, left: &Vec<u8>, right: &Vec<u8>, height: u64, base: &Vec<u8>, split: &Vec<u8>,
                  interior_hash: &Vec<u8>,
                  default_hashes: &Vec<Vec<u8>>){
        let h_str = height.to_string();
        let key = format!("{}{}", h_str, base.to_hex());
        if !default_hashes[height as usize -1].eq(left)  && !default_hashes[height as usize -1].eq(right){
            self.map.insert(key,interior_hash.to_vec());
        }else{
            self.map.remove(&key);
        }
    }
    fn entries(&self) -> usize {
        self.map.len()
    }
}