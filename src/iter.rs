use std::u32::MAX as UMAX;
use rand::Rng;
use super::ShuffledIter as SI;

/// This trait may be used to obtain an instance of `ShuffledIter`. It should
/// always be implemented generically for all types implementing `rand::Rng`,
/// that is for all random number generators (which is the case for this library).
///
/// As such, an instance of `ShuffledIter` may be obtained by calling `iter_shuffled`
/// with a suitable argument on any random number generator, given that `ShuffledIterGen`
/// is in scope.
///
/// An implementation of this trait is provided by this library for the following
/// agument types:
///
/// * all ranges of integer types (ie. `0i8..`, `..10u16`, `10..20`)
/// * slices of any kind
pub trait ShuffledIterGen<T>: Rng {
    type Iter: Iterator;

    fn iter_shuffled(&mut self, T) -> Self::Iter;
}

#[derive(Copy, Clone)]
struct Slice<'a, T: 'a> {
    iter: SI,
    slice: &'a [T],
}

impl<'a, T: 'a, R: Rng> ShuffledIterGen<&'a [T]> for R {
    type Iter = Slice<'a, T>;

    fn iter_shuffled(&mut self, slice: &'a [T]) -> Slice<'a, T> {
        let len = slice.len();

        if len == 0 { panic!("slice must not be empty") }
        if len > UMAX as usize + 1 { panic!("slice may contain at most u32::MAX + 1 elements") }

        Slice {
            iter: SI::new((len - 1) as u32, self),
            slice: slice,
        }
    }
}

impl<'a, T> Iterator for Slice<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        self.iter.next().map(|i| &self.slice[i as usize])
    }
}

macro_rules! impl_range {($prim:ident) => {
    mod $prim {
        #[derive(Copy, Clone)]
        pub struct Iter {
            iter: ::ShuffledIter,
            offset: $prim,
        }

        pub fn new<R: ::rand::Rng>(rng: &mut R, min: $prim, max: $prim) -> Iter {
            if min > max { panic!("min > max") }

            if (max - min) as u64 > ::std::u32::MAX as u64 { panic!("range must contain at most u32::MAX + 1 elements") }

            Iter {
                iter: ::ShuffledIter::new((max - min) as u32, rng),
                offset: min,
            }
        }

        impl Iterator for Iter {
            type Item = $prim;

            #[inline]
            fn next(&mut self) -> Option<$prim> {
                self.iter.next().map(|u| u as $prim + self.offset)
            }
        }
    }

    impl<R: Rng> ShuffledIterGen<::std::ops::Range<$prim>> for R {
        type Iter = $prim::Iter;

        fn iter_shuffled(&mut self, range: ::std::ops::Range<$prim>) -> $prim::Iter {
            if range.end <= range.start { panic!("range must contain at least one element") }
            $prim::new(self, range.start, range.end - 1)
        }
    }

    impl<R: Rng> ShuffledIterGen<::std::ops::RangeFrom<$prim>> for R {
        type Iter = $prim::Iter;

        fn iter_shuffled(&mut self, range: ::std::ops::RangeFrom<$prim>) -> $prim::Iter {
            $prim::new(self, range.start, ::std::$prim::MAX)
        }
    }

    impl<R: Rng> ShuffledIterGen<::std::ops::RangeTo<$prim>> for R {
        type Iter = $prim::Iter;

        fn iter_shuffled(&mut self, range: ::std::ops::RangeTo<$prim>) -> $prim::Iter {
            if range.end == ::std::$prim::MIN { panic!("range must contain at least one element") }
            $prim::new(self, ::std::$prim::MIN, range.end - 1)
        }
    }
}}

macro_rules! impl_ranges {($($prim:ident),+) => {$(
    impl_range!($prim);
)*}}

impl_ranges!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

#[cfg(test)]
#[path = "iter_tests.rs"]
mod iter_tests;
