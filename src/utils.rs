pub fn bit_is_set(bits: &[u8], i: u64) -> bool {
    bits[(i as usize) / 8] & (1 << (7 - i % 8)) != 0
}

pub fn bit_set(bits: &mut [u8], i: u64) {
    bits[(i as usize) / 8] |= 1 << (7 - i % 8)
}

pub fn bit_split(bits: &[u8], i: u64) -> Vec<u8> {
//    let mut split = bits.clone();
    let mut split = bits.to_vec();
    bit_set(&mut split, i);
    split.to_vec()
}