#![feature(test)]

extern crate test;

use smallvec::SmallVec;
use test::{black_box, Bencher};

// Includes constructing and dropping the vector. `reserve` separates tail
// movement from growth; the other cases exercise inline-to-heap transitions.
fn insert_many<T: Copy>(
    b: &mut Bencher,
    item: T,
    tail: usize,
    added: usize,
    index: usize,
    reserve: bool,
    filtered: bool,
) {
    let source = vec![item; tail];
    let input = vec![item; added];
    b.iter(|| {
        let mut v = SmallVec::<[T; 8]>::from_slice(black_box(&source));
        if reserve {
            v.reserve(added);
        }
        let iter = black_box(&input).iter().cloned();
        if filtered {
            v.insert_many(index, iter.filter(|_| true));
        } else {
            v.insert_many(index, iter);
        }
        black_box(v)
    });
}

macro_rules! cases {
    ($($name:ident: $item:expr, $tail:expr, $added:expr, $index:expr, $reserve:expr, $filtered:expr;)*) => {
        $(
            #[bench]
            fn $name(b: &mut Bencher) {
                insert_many(b, $item, $tail, $added, $index, $reserve, $filtered);
            }
        )*
    };
}

cases! {
    filtered_empty: 1u64, 8, 0, 0, false, true;
    filtered_single: 1u64, 7, 1, 0, false, true;
    filtered_pair: 1u64, 6, 2, 0, false, true;
    filtered_inline: 1u64, 4, 4, 0, false, true;
    filtered_spill: 1u64, 8, 8, 0, false, true;
    filtered_heap_growth: 1u64, 64, 64, 0, false, true;
    filtered_front_64: 1u64, 64, 64, 0, true, true;
    filtered_front_1024: 1u64, 1024, 1024, 0, true, true;
    filtered_front_4096: 1u64, 4096, 4096, 0, true, true;
    filtered_middle_1024: 1u64, 1024, 1024, 512, true, true;
    filtered_append_1024: 1u64, 1024, 1024, 1024, true, true;
    exact_inline: 1u64, 4, 4, 0, false, false;
    exact_front_1024: 1u64, 1024, 1024, 0, true, false;
    exact_append_1024: 1u64, 1024, 1024, 1024, true, false;
    filtered_large_elements: [1u64; 32], 64, 64, 0, true, true;
    exact_large_elements: [1u64; 32], 64, 64, 0, true, false;
}

struct Hint<I> {
    iter: I,
    lower: usize,
    upper: bool,
}

impl<I: Iterator> Iterator for Hint<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();
        (lower.min(self.lower), if self.upper { upper } else { None })
    }
}

fn insert_with_hint(b: &mut Bencher, added: usize, lower: usize, upper: bool) {
    let source = [1u64; 6];
    let input = vec![2u64; added];
    b.iter(|| {
        let mut v = SmallVec::<[u64; 8]>::from_slice(black_box(&source));
        v.insert_many(
            0,
            Hint {
                iter: black_box(&input).iter().cloned(),
                lower,
                upper,
            },
        );
        black_box(v)
    });
}

#[bench]
fn unknown_empty(b: &mut Bencher) {
    insert_with_hint(b, 0, 0, false);
}

#[bench]
fn unknown_single(b: &mut Bencher) {
    insert_with_hint(b, 1, 0, false);
}

#[bench]
fn weak_hint_single_excess(b: &mut Bencher) {
    insert_with_hint(b, 2, 1, true);
}

#[bench]
fn weak_hint_multiple_excess(b: &mut Bencher) {
    insert_with_hint(b, 3, 1, true);
}
