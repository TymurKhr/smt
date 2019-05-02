use crate::cache::Cache;
use byteorder::{BigEndian, WriteBytesExt};
use crate::utils::{*};

const EMPTY: [u8; 1] = [0];
const SET: [u8; 1] = [1];

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Key {
    keys: Vec<Vec<u8>>
}

impl Key {
    fn from_vec(keys: Vec<Vec<u8>>) -> Key {
        let mut to_return = Key { keys };
        to_return.sort();
        to_return
    }
    fn from_vec_sorted(keys: Vec<Vec<u8>>) -> Key {
        return Key { keys };
    }
    fn sort(&mut self) {
        self.keys.sort();
    }
    fn split(&self, s: &Vec<u8>) -> (Key, Key) {
        let res = self.keys.binary_search(s);
        let index = match res {
            Ok(n) => n,
            Err(n) => n,
        };
        let (left, right) = self.keys.split_at(index);
        return (Key::from_vec(left.to_vec()), Key::from_vec(right.to_vec()));
    }
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct D {
    d: Vec<Vec<u8>>
}

impl D {
    pub fn from_vec(d: Vec<Vec<u8>>) -> D {
        let mut to_return = D { d };
        to_return.sort();
        to_return
    }
    fn from_vec_sorted(d: Vec<Vec<u8>>) -> D {
        return D { d };
    }
    fn sort(&mut self) {
        self.d.sort();
    }
    pub fn split(&self, s: &Vec<u8>) -> (D, D) {
        let res = self.d.binary_search(s);
        let index = match res {
            Ok(n) => n,
            Err(n) => n,
        };
        let (left, right) = self.d.split_at(index);
        return (D::from_vec_sorted(left.to_vec()), D::from_vec_sorted(right.to_vec()));
    }
}

pub struct SMT<'a, C: Cache> {
    c: Vec<u8>,
    base: Vec<u8>,
    n: u64,
    default_hashes: Vec<Vec<u8>>,
    hash: fn(&Vec<u8>) -> Vec<u8>,
    cache: &'a mut C,
}

impl<'a, C: Cache> SMT<'a, C> {
    fn leaf_hash(&self, a: &Vec<u8>, base: &Vec<u8>) -> Vec<u8> {
        return _leaf_hash(self.hash, &self.c, a, base);
    }

    fn default_hash(&self, height: u64) -> Vec<u8> {
        return self.default_hashes[height as usize].clone();
    }

    fn interior_hash(&self, left: &Vec<u8>, right: &Vec<u8>, height: u64, base: &Vec<u8>) -> Vec<u8> {
        _interior_hash(&self.hash, left, right, height, base)
    }

    pub fn root_hash(&self, d: D, height: u64, base: &Vec<u8>) -> Vec<u8> {
        if self.cache.exists(height, base) {
            return self.cache.get(height, base);
        }
        if d.d.len() == 0 {
            return self.default_hash(height);
        }
        if d.d.len() == 1 && height == 0 {
            return self.leaf_hash(&SET.to_vec(), base);
        }
        if d.d.len() > 0 && height == 0 {
            panic!["this should never happen (unsorted D or broken split)"]
        }

        let split = bit_split(base, self.n - height);
        let (l, r) = d.split(&split);

        return self.interior_hash(
            &self.root_hash(l, height - 1, base),
            &self.root_hash(r, height - 1, &split), height, base);
    }

    pub fn update(&mut self, d: D, key: Key, height: u64, base: &Vec<u8>, value: &Vec<u8>) -> Vec<u8> {
        if height == 0 {
            return self.leaf_hash(value, base);
        }

        let split = bit_split(base, self.n - height);
        let (l_d, r_d) = d.split(&split);
        let (l_keys, r_keys) = key.split(&split);

        if l_keys.keys.len() == 0 && r_keys.keys.len() > 0 {
            return self.cache.hash_cache(
                &self.hash,
                &self.root_hash(l_d, height - 1, base),
                &self.update(r_d, key, height - 1, &split, value),
                height, base, &split, &_interior_hash, &self.default_hashes);
        }
        if l_keys.keys.len() > 0 && r_keys.keys.len() == 0 {
            return self.cache.hash_cache(
                &self.hash,
                &self.update(l_d, key, height - 1, base, value),
                &self.root_hash(r_d, height - 1, &split),
                height, base, &split, &_interior_hash, &self.default_hashes);
        }
        return self.cache.hash_cache(
            &self.hash,
            &self.update(l_d, l_keys, height - 1, base, value),
            &self.update(r_d, r_keys, height - 1, &split, value),
            height, base, &split, &_interior_hash, &self.default_hashes);
    }
    pub fn new(c: Vec<u8>, cache: &'a mut C, hash: Box<fn(&Vec<u8>) -> Vec<u8>>) -> SMT<'a, C> {
        let n = ((*hash)(&Vec::from("abc")).len() * 8) as u64;
        let mut default_hashes: Vec<Vec<u8>> = Vec::new();
        default_hashes.push(_leaf_hash(*hash, &c, &EMPTY.to_vec(), &Vec::new()));
        for i in 1..(n + 1) {
            let prev_index = (i - 1) as usize;
            default_hashes.push(hash(&cat(
                default_hashes[prev_index].as_slice(),
                default_hashes[prev_index].as_slice())));
        }

        return SMT {
            c,
            base: vec![0 as u8; (n / 8) as usize],
            n,
            default_hashes,
            hash: *hash,
            cache,
        };
    }
}

fn _interior_hash(hash: &Fn(&Vec<u8>) -> Vec<u8>, left: &Vec<u8>, right: &Vec<u8>, height: u64, base: &Vec<u8>) -> Vec<u8> {
    let lr = cat(left, right);
    if left == right {
        return hash(&lr);
    }
    let mut height_serialized = vec![];
    height_serialized.write_u64::<BigEndian>(height).expect("failed to serialize height");
    let lr = cat(&lr, base);
    let lr = cat(&lr, &height_serialized);

    return hash(&lr);
}

fn _leaf_hash(hash: fn(&Vec<u8>) -> Vec<u8>, c: &Vec<u8>, a: &Vec<u8>, base: &Vec<u8>) -> Vec<u8> {
    if *a == EMPTY {
        return hash(c);
    }
    let mut to_hash = c.to_vec();
    to_hash.extend(base);
    return hash(&to_hash);
}

fn cat(a: &[u8], b: &[u8]) -> Vec<u8> {
    [a, b].concat()
}
