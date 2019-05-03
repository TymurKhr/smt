use ring::digest;
use smt::cache::{CacheNothing};
use smt::tree::{SET, EMPTY, Key, D, SMT};
use std::assert_eq;
use rustc_serialize::hex::ToHex;
use smt::cache::CacheBranch;
use std::collections::HashMap;

fn sadapter(algorithm: &'static digest::Algorithm) -> Box<Fn(&Vec<u8>)->Vec<u8>>{
    let closure = move |x:&Vec<u8>|  digest::digest(algorithm, x.as_slice()).as_ref().to_vec();
    return Box::new(closure);
}
fn main() {
    let smt = SMT::new(vec![0x42 as u8], sadapter(&digest::SHA256));
    let mut key: Vec<Vec<u8>>= Vec::new();
    key.push((*sadapter(&digest::SHA256))(&"abc".as_bytes().to_vec()));
    key.push((*sadapter(&digest::SHA256))(&"bcde".as_bytes().to_vec()));

    let keys = Key::from_vec(key.clone());
    let d = D::from_vec(key.clone());
    let mut c = CacheNothing{};
    let update_hash = smt.update(&d, keys, &SET.to_vec(), &mut c);
    let root_hash = smt.root_hash(&d, &c);
    assert_eq!(update_hash, root_hash);

    let mut t_key: Vec<u8> =(*sadapter(&digest::SHA256))(&"not_member".as_bytes().to_vec()) ;
    let ap = smt.audit_path(&d,  &t_key, &c);
    assert_eq!(smt.verify_audit_path(&ap, &t_key, &EMPTY.to_vec(), &root_hash), true);
    assert_eq!(smt.verify_audit_path(&ap, &t_key, &SET.to_vec(), &root_hash), false);

    t_key = (*sadapter(&digest::SHA256))(&"abc".as_bytes().to_vec());
    let ap = smt.audit_path(&d, &t_key, &c);
    assert_eq!(smt.verify_audit_path(&ap, &t_key, &EMPTY.to_vec(), &root_hash), false);
    assert_eq!(smt.verify_audit_path(&ap, &t_key, &SET.to_vec(), &root_hash), true);

    let keys = Key::from_vec(key.clone());
    let mut c_b = CacheBranch::new();
    let b_update_hash = smt.update(&d, keys, &SET.to_vec(), &mut c_b);
    let b_root_hash = smt.root_hash(&d, &c);
    assert_eq!(b_update_hash, b_root_hash);
    assert_eq!(update_hash, b_update_hash);

}
