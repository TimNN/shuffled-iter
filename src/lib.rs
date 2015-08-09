//! This crate provides methods to iterate over a group of values in random
//! order, without allocation and shuffling them all.
//!
//! Such an iterator may be obtained via the `ShuffledIterGen` trait.

#[cfg(test)] extern crate bit_vec;
extern crate rand;

pub use iter::ShuffledIterGen;

use std::num::Wrapping;
use rand::Rng;

mod iter;

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

/// The actual iterator which iterates over, in random order, all `u32`s
/// smaller than or equal to a given maximum value.
///
/// An instance of this struct may be obtained via the `ShuffletIterGen` trait.
///
/// No gurantees are made about the quality of the randomiziation, nor is the
/// algorithem guranteed to remain unchanged between version.
#[derive(Copy, Clone, Debug)]
pub struct ShuffledIter {
    max: w32,
    mask: w32,

    index: w32,
    count: w32,

    f1: w32,
    f2: w32,
    x1: w32,
    x2: w32,

    done: bool,
}

impl ShuffledIter {
    fn new<R: Rng>(max: u32, rng: &mut R) -> ShuffledIter {
        let bits = 32 - max.leading_zeros();

        let max = w32(max);

        let mask = w32(shl_ignore(1, bits)) - w32(1);

        ShuffledIter {
            max: max,
            mask: mask,

            index: w32(rng.next_u32()),
            count: w32(0),

            f1: gen_factor(rng),
            f2: gen_factor(rng),
            x1: gen_xor_op(rng),
            x2: gen_xor_op(rng),

            done: false,
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

impl Iterator for ShuffledIter {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<u32> {
        if self.count < self.max {
            Some(self.next_value())
        } else if self.done {
            None
        } else {
            self.done = true;
            self.count = self.count - w32(1);
            // if max == u32::max, next_value would wrap to 0, triggering the first block again
            Some(self.next_value())
        }
    }
}

#[test]
fn gen_1_1024() {
    use bit_vec::BitVec;
    use rand::XorShiftRng;

    let mut rng = XorShiftRng::new_unseeded();

    for i in 0u32 .. 1025 {
        let mut bv = BitVec::from_elem(i as usize + 1, false);

        let it = ShuffledIter::new(i, &mut rng);

        for j in it {
            assert!(bv.get(j as usize) == Some(false));
            bv.set(j as usize, true);
        }

        assert!(bv.all());
    }
}

#[test]
fn max_u32_max() {
    use std::u32;
    use rand::XorShiftRng;

    let mut rng = XorShiftRng::new_unseeded();

    let mut it = ShuffledIter::new(u32::MAX, &mut rng);

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
