/// 64-bits bitsfield
pub struct Bitsfield {
    data1: u32,
    data2: u32,
}

impl Bitsfield {
    pub fn new() -> Bitsfield {
        Bitsfield {
            data1: 0xffffffff,
            data2: 0xffffffff,
        }
    }

    pub fn set_used(&mut self, mut bit: u8) {
        if bit < 32 {
            let mask: u32 = 1 << bit;
            let mask = !mask;
            self.data1 &= mask;
            return;
        }

        bit -= 32;

        if bit < 32 {
            let mask: u32 = 1 << bit;
            let mask = !mask;
            self.data2 &= mask;
            return;
        }

        unimplemented!();
    }

    pub fn set_unused(&mut self, mut bit: u8) {
        if bit < 32 {
            let mask: u32 = 1 << bit;
            self.data1 |= mask;
            return;
        }

        bit -= 32;

        if bit < 32 {
            let mask: u32 = 1 << bit;
            self.data2 |= mask;
            return;
        }

        unimplemented!();
    }

    pub fn is_used(&self, mut bit: u8) -> bool {
        if bit < 32 {
            let mask: u32 = 1 << bit;
            return self.data1 & mask == 0;
        }

        bit -= 32;

        if bit < 32 {
            let mask: u32 = 1 << bit;
            return self.data2 & mask == 0;
        }

        unimplemented!();
    }

    pub fn get_unused(&self) -> Option<u8> {
        let v = self.data1.trailing_zeros() as u8;
        if v < 32 {
            return Some(v);
        }

        let v = self.data2.trailing_zeros() as u8;
        if v < 32 {
            return Some(v + 32);
        }

        None
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
        for i in (0 .. 34) {
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
