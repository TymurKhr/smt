use crate::cache::Cache;
use byteorder::{BigEndian, WriteBytesExt};
use crate::utils::{*};

pub const EMPTY: [u8; 1] = [0];
pub const SET: [u8; 1] = [1];

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Key {
    keys: Vec<Vec<u8>>
}

impl Key {
    pub fn from_vec(keys: Vec<Vec<u8>>) -> Key {
        let mut to_return = Key { keys };
        to_return.sort();
        to_return
    }
    fn from_vec_sorted(keys: Vec<Vec<u8>>) -> Key {
        Key { keys }
    }
    fn sort(&mut self) {
        self.keys.sort();
    }
    fn split(&self, s: &[u8]) -> (Key, Key) {
        let res = self.keys.binary_search(&s.to_vec());
        let index = match res {
            Ok(n) => n,
            Err(n) => n,
        };
        let (left, right) = self.keys.split_at(index);
        (Key::from_vec_sorted(left.to_vec()), Key::from_vec_sorted(right.to_vec()))
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
        D { d }
    }
    fn sort(&mut self) {
        self.d.sort();
    }
    pub fn split(&self, s: &[u8]) -> (D, D) {
        let res = self.d.binary_search(&s.to_vec());
        let index = match res {
            Ok(n) => n,
            Err(n) => n,
        };
        let (left, right) = self.d.split_at(index);
        (D::from_vec_sorted(left.to_vec()), D::from_vec_sorted(right.to_vec()))
    }
}

pub struct SMT {
    c: Vec<u8>,
    pub base: Vec<u8>,
    pub n: u64,
    default_hashes: Vec<Vec<u8>>,
    hash: Box<Fn(&[u8]) -> Vec<u8>>,
}

impl SMT {
    pub fn new(c: Vec<u8>, hash: Box<Fn(&[u8]) -> Vec<u8>>) -> SMT {
        let n = ((*hash)(&Vec::from("abc")).len() * 8) as u64;
        let mut default_hashes: Vec<Vec<u8>> = Vec::new();
        default_hashes.push(_leaf_hash(&hash, &c, &EMPTY.to_vec(), &Vec::new()));
        for i in 1..(n + 1) {
            let prev_index = (i - 1) as usize;
            default_hashes.push(hash(&cat(
                default_hashes[prev_index].as_slice(),
                default_hashes[prev_index].as_slice())));
        }

       SMT {
            c,
            base: vec![0 as u8; (n / 8) as usize],
            n,
            default_hashes,
            hash,
       }
    }

    pub fn audit_path<C: Cache>(&self, d: &D, key: &[u8], cache: &C) -> Vec<Vec<u8>> {
        self.audit_path_internal(d, self.n, &self.base, key, cache)
    }

    fn audit_path_internal<C: Cache>(&self, d: &D, height: u64, base: &[u8], key: &[u8], cache: &C) -> Vec<Vec<u8>> {
        if height == 0 {
            return Vec::new();
        }
        let split = bit_split(base, self.n - height);
        let (l, r) = d.split(&split);

        if !bit_is_set(key, self.n - height) {
            let mut t = self.audit_path_internal(&l, height - 1, base, key, cache);
            t.push(self.root_hash_internal(&r, height - 1, &split, cache));
            return t;
        }
        let mut t = self.audit_path_internal(&r, height - 1, &split, key, cache);
        t.push(self.root_hash_internal(&l, height - 1, base, cache));
        t
    }

    fn audit_path_calc(&self, ap: &Vec<Vec<u8>>, height: u64, base: &[u8], key: &[u8], value: &[u8]) -> Vec<u8> {
        if height == 0 {
            return self.leaf_hash(value, base);
        }
        let split = bit_split(base, self.n - height);
        if !bit_is_set(key, self.n - height) {
            return self.interior_hash(
                &self.audit_path_calc(ap, height - 1, base, key, value),
                &ap[height as usize - 1],
                height,
                base,
            );
        }

        self.interior_hash(
            &ap[height as usize - 1],
            &self.audit_path_calc(ap, height - 1, &split, key, value),
            height,
            base,
        )
    }

    pub fn verify_audit_path(&self, ap: &Vec<Vec<u8>>, key: &[u8], value: &[u8], root: &[u8]) -> bool {
        root.eq(self.audit_path_calc(ap, self.n, &vec![0 as u8; (self.n / 8) as usize], key, value).as_slice())
    }

    pub fn root_hash<C: Cache>(&self, d: &D, cache: &C) -> Vec<u8>{
        self.root_hash_internal(d, self.n, &self.base, cache)
    }

    fn root_hash_internal<C: Cache>(&self, d: &D, height: u64, base: &[u8], cache: &C) -> Vec<u8> {
        if cache.exists(height, base) {
            return cache.get(height, base);
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

        self.interior_hash(
            &self.root_hash_internal(&l, height - 1, base, cache),
            &self.root_hash_internal(&r, height - 1, &split, cache), height, base)
    }

    pub fn update<C: Cache>(&self, d: &D, key: Key, value: &[u8], cache: &mut C) -> Vec<u8> {
        self.update_internal(d, key, self.n, &self.base, value, cache)
    }

    fn update_internal<C: Cache>(&self, d: &D, key: Key, height: u64, base: &[u8], value: &[u8], cache: &mut C) -> Vec<u8> {
        if height == 0 {
            return self.leaf_hash(value, base);
        }

        let split = bit_split(base, self.n - height);
        let (l_d, r_d) = d.split(&split);
        let (l_keys, r_keys) = key.split(&split);

        if l_keys.keys.len() == 0 && r_keys.keys.len() > 0 {
            let left = &self.root_hash_internal(&l_d, height - 1, base, cache);
            let right = &self.update_internal(&r_d, key, height - 1, &split, value, cache);
            let ih = self.interior_hash(left, right, height, base);
            cache.hash_cache(left, right, height, base, &split, &ih, &self.default_hashes);
            return ih;
        }
        if l_keys.keys.len() > 0 && r_keys.keys.len() == 0 {
            let left = &self.update_internal(&l_d, key, height - 1, base, value, cache);
            let right = &self.root_hash_internal(&r_d, height - 1, &split, cache);
            let ih = self.interior_hash(left, right, height, base);
            cache.hash_cache(left, right, height, base, &split, &ih, &self.default_hashes);
            return ih;
        }

        let left = &self.update_internal(&l_d, l_keys, height - 1, base, value, cache);
        let right = &self.update_internal(&r_d, r_keys, height - 1, &split, value, cache);
        let ih = self.interior_hash(left, right, height, base);
        cache.hash_cache(left, right, height, base, &split, &ih, &self.default_hashes);

        ih
    }


    fn leaf_hash(&self, a: &[u8], base: &[u8]) -> Vec<u8> {
        _leaf_hash(&self.hash, &self.c, a, base)
    }

    fn default_hash(&self, height: u64) -> Vec<u8> {
        self.default_hashes[height as usize].clone()
    }

    fn interior_hash(&self, left: &[u8], right: &[u8], height: u64, base: &[u8]) -> Vec<u8> {
        _interior_hash(&self.hash, left, right, height, base)
    }
}

fn _interior_hash(hash: &Box<Fn(&[u8]) -> Vec<u8>>, left: &[u8], right: &[u8], height: u64, base: &[u8]) -> Vec<u8> {
    let lr = cat(left, right);
    if left == right {
        return hash(&lr);
    }
    let mut height_serialized = vec![];
    height_serialized.write_u64::<BigEndian>(height).expect("failed to serialize height");
    let lr = cat(&lr, base);
    let lr = cat(&lr, &height_serialized);

    hash(&lr)
}

fn _leaf_hash(hash: &Box<Fn(&[u8]) -> Vec<u8>>, c: &[u8], a: &[u8], base: &[u8]) -> Vec<u8> {
    if *a == EMPTY {
        return hash(c);
    }
    let mut to_hash = c.to_vec();
    to_hash.extend(base);
    hash(&to_hash)
}

fn cat(a: &[u8], b: &[u8]) -> Vec<u8> {
    [a, b].concat()
}
