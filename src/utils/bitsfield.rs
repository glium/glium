const NUM_DWORDS: usize = 8;

/// 64-bits bitsfield
pub struct Bitsfield {
    data: [u32; NUM_DWORDS],
}

impl Bitsfield {
    #[inline]
    pub fn new() -> Bitsfield {
        Bitsfield {
            data: [0xffffffff; NUM_DWORDS],
        }
    }

    #[inline]
    pub fn set_used(&mut self, mut bit: u16) {
        let mut offset = 0;

        loop {
            if offset >= NUM_DWORDS {
                unimplemented!();
            }

            if bit < 32 {
                let mask: u32 = 1 << bit;
                let mask = !mask;
                self.data[offset] &= mask;
                return;
            }

            bit -= 32;
            offset += 1;
        }
    }

    #[inline]
    pub fn set_unused(&mut self, mut bit: u16) {
        let mut offset = 0;

        loop {
            if offset >= NUM_DWORDS {
                unimplemented!();
            }

            if bit < 32 {
                let mask: u32 = 1 << bit;
                self.data[offset] |= mask;
                return;
            }

            bit -= 32;
            offset += 1;
        }
    }

    #[inline]
    pub fn is_used(&self, mut bit: u16) -> bool {
        let mut offset = 0;

        loop {
            if offset >= NUM_DWORDS {
                unimplemented!();
            }

            if bit < 32 {
                let mask: u32 = 1 << bit;
                return self.data[offset] & mask == 0;
            }

            bit -= 32;
            offset += 1;
        }
    }

    #[inline]
    pub fn get_unused(&self) -> Option<u16> {
        let mut offset = 0;

        loop {
            if offset >= NUM_DWORDS {
                return None;
            }

            let v = self.data[offset].trailing_zeros() as u16;
            if v < 32 {
                return Some(v + offset as u16 * 32);
            }

            offset += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Bitsfield;

    #[test]
    fn get_unused() {
        let mut bitsfield = Bitsfield::new();
        bitsfield.set_used(0);
        bitsfield.set_used(1);
        assert_eq!(bitsfield.get_unused(), Some(2))
    }

    #[test]
    fn get_unused2() {
        let mut bitsfield = Bitsfield::new();
        for i in 0 .. 34 {
            bitsfield.set_used(i);
        }
        assert_eq!(bitsfield.get_unused(), Some(34))
    }

    #[test]
    fn is_used() {
        let mut bitsfield = Bitsfield::new();
        bitsfield.set_used(5);
        bitsfield.set_used(6);
        bitsfield.set_used(37);
        assert!(!bitsfield.is_used(4));
        assert!(bitsfield.is_used(5));
        assert!(bitsfield.is_used(6));
        assert!(!bitsfield.is_used(7));
        assert!(!bitsfield.is_used(36));
        assert!(bitsfield.is_used(37));
        assert!(!bitsfield.is_used(38));
    }
}
