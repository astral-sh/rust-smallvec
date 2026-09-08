use {
    smallvec::SmallVec,
    std::{
        cell::Cell,
        rc::Rc
    }
};

fn check_collect<const M: usize, const N: usize>() {
    for len in [0, 1, 7, 8, 9, 16, 32] {
        for (front, back) in [
            (0, 0),
            (len / 2, 0),
            (0, len / 2),
            (len / 3, len / 3),
            (len, 0)
        ] {
            for spilled in [false, true] {
                let source: SmallVec<usize, M> = if spilled {
                    SmallVec::from_vec((0..len).collect())
                } else {
                    (0..len).collect()
                };
                let source_ptr = source.as_ptr();
                let source_cap = source.capacity();
                let source_spilled = source.spilled();
                let mut iter = source.into_iter();
                for _ in 0..front {
                    iter.next();
                }
                for _ in 0..back {
                    iter.next_back();
                }
                let result: SmallVec<usize, N> = iter.collect();
                assert_eq!(&result[..], &(front..len - back).collect::<Vec<_>>());
                assert_eq!(result.spilled(), result.len() > N);
                if cfg!(feature = "specialization") && source_spilled && result.len() > N {
                    assert_eq!(result.as_ptr(), source_ptr);
                    assert_eq!(result.capacity(), source_cap);
                }
            }
        }
    }
}

#[test]
fn collect_across_inline_capacities() {
    check_collect::<8, 8>();
    check_collect::<8, 16>();
    check_collect::<16, 8>();
    check_collect::<0, 8>();
    check_collect::<8, 0>();
}

#[test]
fn collect_zst() {
    let source: SmallVec<(), 8> = std::iter::repeat_n((), 32).collect();
    let mut iter = source.into_iter();
    iter.next();
    iter.next_back();
    let result: SmallVec<(), 0> = iter.collect();
    assert_eq!(result.len(), 30);
    assert!(!result.spilled());
}

struct Tracked(Rc<Cell<usize>>);

impl Drop for Tracked {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn collect_drops_every_element_once() {
    for (front, back) in [(0, 0), (3, 2), (8, 0), (10, 0)] {
        let drops: Vec<_> = (0..10).map(|_| Rc::new(Cell::new(0))).collect();
        let source: SmallVec<Tracked, 4> = drops.iter().cloned().map(Tracked).collect();
        let mut iter = source.into_iter();
        for _ in 0..front {
            drop(iter.next());
        }
        for _ in 0..back {
            drop(iter.next_back());
        }
        let result: SmallVec<Tracked, 4> = iter.collect();
        for (index, count) in drops.iter().enumerate() {
            assert_eq!(
                count.get(),
                usize::from(index < front || index >= 10 - back)
            );
        }
        drop(result);
        assert!(drops.iter().all(|count| count.get() == 1));
    }
}
