//! This module proivdes an iterator which iterates through all elements in a
//! sequence in random order without allocation each element and then shuffling.

#[cfg(test)] extern crate bit_vec;
extern crate rand;

use std::marker::PhantomData;
use std::num::Wrapping;

use rand::Rng;

#[allow(non_camel_case_types)]
type w32 = Wrapping<u32>;

#[inline(always)]
fn w32(u: u32) -> w32 {
    Wrapping(u)
}

#[inline(always)]
fn shl_ignore(slf: u32, rhs: u32) -> u32 {
    if rhs >= 32 { 0 } else { slf << rhs }
}

// This needs to be an odd number so that the multiplicative inverse of the
// result modulo 2^n exists
// ie.: gcd(2^n, res) == 1
#[inline(always)]
fn gen_factor<R: Rng>(rng: &mut R) -> w32 {
    w32((rng.next_u32() << 1) | 1)
}

// No restricitions here
#[inline(always)]
fn gen_xor_op<R: Rng>(rng: &mut R) -> w32 {
    w32(rng.next_u32())
}

#[derive(Debug)]
pub struct ShuffledSeqIter<T = u32> where u32: Into<T> {
    max: w32,
    mask: w32,

    index: w32,
    count: w32,

    f1: w32,
    f2: w32,
    x1: w32,
    x2: w32,

    done: bool,

    _phantom: PhantomData<T>,
}

impl<T> ShuffledSeqIter<T> where u32: Into<T> {
    pub fn new<R: Rng>(max: u32, rng: &mut R) -> ShuffledSeqIter<T> {
        assert!(max > 0);

        let bits = 32 - max.leading_zeros();

        let max = w32(max);

        let mask = w32(shl_ignore(1, bits)) - w32(1);

        ShuffledSeqIter {
            max: max,
            mask: mask,

            index: w32(rng.next_u32()),
            count: w32(0),

            f1: gen_factor(rng),
            f2: gen_factor(rng),
            x1: gen_xor_op(rng),
            x2: gen_xor_op(rng),

            done: false,

            _phantom: PhantomData,
        }
    }

    #[inline(always)]
    fn next_value(&mut self) -> u32 {
        self.index = self.index + w32(1);
        self.count = self.count + w32(1);

        let mut val = self.calc(self.index);

        while val > self.max {
            self.index = self.index + w32(1);
            val = self.calc(self.index);
        }

        val.0
    }

    #[inline(always)]
    fn calc(&self, val: w32) -> w32 {
        ((((val * self.f1) ^ self.x1) * self.f2) ^ self.x2) & self.mask
    }
}

impl<T> Iterator for ShuffledSeqIter<T> where u32: Into<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        if self.count < self.max {
            Some(self.next_value().into())
        } else if self.done {
            None
        } else {
            self.done = true;
            self.count = self.count - w32(1);
            // if max == u32::max, next_value would wrap to 0, triggering the first block again
            Some(self.next_value().into())
        }
    }
}


#[test]
fn gen_1_1024() {
    use bit_vec::BitVec;
    use rand::XorShiftRng;

    let mut rng = XorShiftRng::new_unseeded();

    for i in 1u32 .. 1025 {
        let mut bv = BitVec::from_elem(i as usize + 1, false);

        let it: ShuffledSeqIter<u32> = ShuffledSeqIter::new(i, &mut rng);

        for j in it {
            assert!(bv.get(j as usize) == Some(false));
            bv.set(j as usize, true);
        }

        assert!(bv.all());
    }
}

#[test]
fn u32_max() {
    use std::u32;
    use rand::XorShiftRng;

    let mut rng = XorShiftRng::new_unseeded();

    let mut it: ShuffledSeqIter<u32> = ShuffledSeqIter::new(u32::MAX, &mut rng);

    for _ in 0 .. 10 {
        assert!(it.next().is_some());
    }

    it.count = w32(u32::MAX - 9);
    // We now should have 10 values left (u32::MAX plus 9 others)

    for _ in 0 .. 10 {
        assert!(it.next().is_some());
    }

    println!("{}", it.count.0);

    assert!(it.next().is_none());
}
