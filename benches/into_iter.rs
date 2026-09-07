#![feature(test)]

extern crate test;

use smallvec::SmallVec;
use test::{black_box, Bencher};

#[derive(Clone)]
struct Dropped(u64);

impl Drop for Dropped {
    fn drop(&mut self) {
        black_box(self.0);
    }
}

// Construction and destruction are included in libtest's timing. `next` is a
// control with the same construction cost and no skipped elements.
macro_rules! cases {
    ($name:ident, $len:expr, $value:expr) => {
        mod $name {
            use super::*;

            #[bench]
            fn nth(b: &mut Bencher) {
                let source: SmallVec<[_; 16]> = (0..$len).map(|_| $value).collect();
                b.iter(|| {
                    let mut iter = black_box(source.clone()).into_iter();
                    black_box(iter.nth(black_box($len.saturating_sub(1))))
                });
            }

            #[bench]
            fn nth_back(b: &mut Bencher) {
                let source: SmallVec<[_; 16]> = (0..$len).map(|_| $value).collect();
                b.iter(|| {
                    let mut iter = black_box(source.clone()).into_iter();
                    black_box(iter.nth_back(black_box($len.saturating_sub(1))))
                });
            }

            #[bench]
            fn next(b: &mut Bencher) {
                let source: SmallVec<[_; 16]> = (0..$len).map(|_| $value).collect();
                b.iter(|| black_box(black_box(source.clone()).into_iter().next()));
            }

            #[bench]
            fn vec_nth(b: &mut Bencher) {
                let source: Vec<_> = (0..$len).map(|_| $value).collect();
                b.iter(|| {
                    black_box(
                        black_box(source.clone())
                            .into_iter()
                            .nth(black_box($len.saturating_sub(1))),
                    )
                });
            }
        }
    };
}

cases!(empty, 0usize, 7u64);
cases!(inline, 8usize, 7u64);
cases!(boundary, 16usize, 7u64);
cases!(first_spill, 17usize, 7u64);
cases!(spilled, 4096usize, 7u64);
cases!(large_items, 256usize, [7u64; 16]);
cases!(drop_items, 256usize, Dropped(7));
cases!(zero_sized, 4096usize, ());
