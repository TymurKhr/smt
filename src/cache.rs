use std::collections::HashMap;
use rustc_serialize::hex::ToHex;

pub trait Cache {
    fn exists(&self, height: u64, base: &[u8]) -> bool;
    fn get(&self, height: u64, base: &[u8]) -> Vec<u8>;
    fn hash_cache(&mut self, left: &[u8], right: &[u8], height: u64, base: &[u8], split: &[u8],
                  interior_hash: &[u8],
                  default_hashes: &Vec<Vec<u8>>);
    fn entries(&self) -> usize;
}

pub struct CacheNothing;

impl Cache for CacheNothing {
    fn exists(&self, _height: u64, _base: &[u8]) -> bool {
        false
    }
    fn get(&self, _height: u64, _base: &[u8]) -> Vec<u8> {
        Vec::new()
    }
    fn hash_cache(&mut self, _left: &[u8], _right: &[u8], _height: u64, _base: &[u8], _split: &[u8],
                  _interior_hash: &[u8],
                  _default_hashes: &Vec<Vec<u8>>){
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
    fn exists(&self, height: u64, base: &[u8]) -> bool {
        let h_str = height.to_string();
        let key = format!("{}{}", h_str, base.to_hex());
        self.map.get(&key).is_some()
    }
    fn get(&self, height: u64, base: &[u8]) -> Vec<u8> {
        let h_str = height.to_string();
        let key = format!("{}{}", h_str, base.to_hex());
        self.map.get(&key).expect("get should be used only after exists").clone()
    }
    fn hash_cache(&mut self, left: &[u8], right: &[u8], height: u64, base: &[u8], split: &[u8],
                  interior_hash: &[u8],
                  default_hashes: &Vec<Vec<u8>>){
        let h_str = height.to_string();
        let key = format!("{}{}", h_str, base.to_hex());
        if !default_hashes[height as usize -1].as_slice().eq(left)  && !default_hashes[height as usize -1].as_slice().eq(right){
            self.map.insert(key,interior_hash.to_vec());
        }else{
            self.map.remove(&key);
        }
    }
    fn entries(&self) -> usize {
        self.map.len()
    }
}