pub trait BitSet {
    fn set_bit(&mut self, bit: usize);
    fn unset_bit(&mut self, bit: usize);
    fn bit(&self, bit: usize) -> bool;
    fn clear_bits(&mut self);
}

impl BitSet for [u8] {
    fn set_bit(&mut self, bit: usize) {
        let byte = bit >> 3;
        let bit = bit & 7;
        self[byte] |= 1 << bit;
    }

    fn unset_bit(&mut self, bit: usize) {
        let byte = bit >> 3;
        let bit = bit & 7;
        self[byte] &= !(1 << bit);
    }

    fn bit(&self, bit: usize) -> bool {
        let byte = bit >> 3;
        let bit = bit & 7;
        self[byte] & (1 << bit) != 0
    }

    fn clear_bits(&mut self) {
        for i in 0..self.len() {
            self[i] = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitset_set_bit() {
        let mut bitset = [0u8; 2];
        bitset.set_bit(0);
        assert_eq!(bitset[0], 1u8);
        assert_eq!(bitset[1], 0u8);
        bitset.set_bit(8);
        assert_eq!(bitset[0], 1u8);
        assert_eq!(bitset[1], 1u8);
        bitset.set_bit(10);
        assert_eq!(bitset[0], 1u8);
        assert_eq!(bitset[1], 5u8);
    }

    #[test]
    fn bitset_bit() {
        let mut bitset = [0u8; 2];
        bitset.set_bit(0);
        bitset.set_bit(8);
        bitset.set_bit(10);
        assert!(bitset.bit(0));
        assert!(bitset.bit(8));
        assert!(bitset.bit(10));
        for i in 0..=15 {
            if i == 0 || i == 8 || i == 10 {
                continue;
            }
            assert!(!bitset.bit(i));
        }
    }
}
