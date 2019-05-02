pub trait Cache {
    fn exists(&self, height: u64, base: &Vec<u8>) -> bool;
    fn get(&self, height: u64, base: &Vec<u8>) -> Vec<u8>;
    fn hash_cache(&self, hash: &Fn(&Vec<u8>) -> Vec<u8>, left: &Vec<u8>, right: &Vec<u8>, height: u64, base: &Vec<u8>, split: &Vec<u8>,
                  _interior_hash: &Fn(&Fn(&Vec<u8>) -> Vec<u8>, &Vec<u8>, &Vec<u8>, u64, &Vec<u8>) -> Vec<u8>,
                  default_hashes: &Vec<Vec<u8>>) -> Vec<u8>;
    fn entries(&self) -> isize;
}

pub type CacheNothing = isize;

impl Cache for CacheNothing {
    fn exists(&self, height: u64, base: &Vec<u8>) -> bool {
        false
    }
    fn get(&self, height: u64, base: &Vec<u8>) -> Vec<u8> {
        Vec::new()
    }
    fn hash_cache(&self, hash: &Fn(&Vec<u8>) -> Vec<u8>, left: &Vec<u8>, right: &Vec<u8>, height: u64, base: &Vec<u8>, split: &Vec<u8>,
                  _interior_hash: &Fn(&Fn(&Vec<u8>) -> Vec<u8>, &Vec<u8>, &Vec<u8>, u64, &Vec<u8>) -> Vec<u8>,
                  default_hashes: &Vec<Vec<u8>>) -> Vec<u8> {
        _interior_hash(hash, left, right, height, base)
    }
    fn entries(&self) -> isize {
        0
    }
}