macro_rules! range_tests {($prim:ident) => {
    mod $prim {
        use iter::ShuffledIterGen;

        fn test(min: $prim, max: $prim, iter: ::iter::$prim::Iter) {
            let cnt = (max - min + 1) as usize;
            let mut bv = ::bit_vec::BitVec::from_elem(cnt, false);

            for i in iter {
                assert!(min <= i && i <= max);

                let i = (i - min) as usize;
                assert!(bv.get(i) == Some(false));
                bv.set(i, true);
            }

            assert!(bv.all());
        }

        #[test]
        fn min() {
            let mut rng = ::rand::XorShiftRng::new_unseeded();
            let iter = rng.iter_shuffled(.. ::std::$prim::MIN + 10);
            test(::std::$prim::MIN, ::std::$prim::MIN + 9, iter);
        }

        #[test]
        fn max() {
            let mut rng = ::rand::XorShiftRng::new_unseeded();
            let iter = rng.iter_shuffled(::std::$prim::MAX - 10 ..);
            test(::std::$prim::MAX - 10, ::std::$prim::MAX, iter);
        }

        #[test]
        fn median() {
            let mut rng = ::rand::XorShiftRng::new_unseeded();
            let median = ::std::$prim::MIN / 2 + ::std::$prim::MAX / 2;
            let iter = rng.iter_shuffled(median - 5 .. median + 5);
            test(median - 5, median + 4, iter);
        }
    }
}}

macro_rules! ranges_tests {($($prim:ident),+) => {$(
    range_tests!($prim);
)*}}

ranges_tests!(u8, u16, u32, u64, usize, i8, i16, i32, isize);
