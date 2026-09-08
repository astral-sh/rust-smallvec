#![feature(test)]

extern crate test;

use smallvec::SmallVec;
use test::{black_box, Bencher};

// Includes creating the owned argument and dropping the result on both paths.
fn from_elem<T: Clone>(b: &mut Bencher, value: T, n: usize) {
    b.iter(|| SmallVec::<[T; 8]>::from_elem(black_box(value.clone()), black_box(n)));
}

fn resize<T: Clone>(b: &mut Bencher, value: T, old_len: usize, added: usize, reserve: bool) {
    let source = SmallVec::<[T; 8]>::from_elem(value.clone(), old_len);
    b.iter(|| {
        let mut v = source.clone();
        if reserve {
            v.reserve(added);
        }
        v.resize(black_box(old_len + added), black_box(value.clone()));
        black_box(v)
    });
}

macro_rules! from_elem_cases {
    ($($name:ident: $value:expr, $n:expr;)*) => {
        $(
            #[bench]
            fn $name(b: &mut Bencher) {
                from_elem(b, $value, $n);
            }
        )*
    };
}

macro_rules! resize_cases {
    ($($name:ident: $value:expr, $old_len:expr, $added:expr, $reserve:expr;)*) => {
        $(
            #[bench]
            fn $name(b: &mut Bencher) {
                resize(b, $value, $old_len, $added, $reserve);
            }
        )*
    };
}

from_elem_cases! {
    from_elem_string_empty: "x".repeat(256), 0;
    from_elem_string_one: "x".repeat(256), 1;
    from_elem_string_inline: "x".repeat(256), 8;
    from_elem_string_spilled_control: "x".repeat(256), 9;
    from_elem_u64_one: 1u64, 1;
    from_elem_u64_inline: 1u64, 8;
    from_elem_u64_spilled_control: 1u64, 9;
}

resize_cases! {
    resize_string_empty: "x".repeat(256), 0, 0, false;
    resize_string_one: "x".repeat(256), 0, 1, false;
    resize_string_inline: "x".repeat(256), 0, 8, false;
    resize_string_spill: "x".repeat(256), 8, 1, false;
    resize_string_heap_growth: "x".repeat(256), 9, 8, false;
    resize_string_heap_reserved: "x".repeat(256), 9, 8, true;
    resize_u64_one: 1u64, 0, 1, false;
    resize_u64_inline: 1u64, 0, 8, false;
    resize_u64_heap_growth: 1u64, 9, 8, false;
}
