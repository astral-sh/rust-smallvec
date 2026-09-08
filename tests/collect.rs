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
                if cfg!(feature = "specialization")
                    && source_spilled
                    && result.len() > N
                    && result.len() >= source_cap.div_ceil(2)
                {
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

#[test]
fn collect_zst_with_drop() {
    thread_local! {
        static DROPS: Cell<usize> = const { Cell::new(0) };
    }
    struct Zst;
    impl Drop for Zst {
        fn drop(&mut self) {
            DROPS.with(|count| count.set(count.get() + 1));
        }
    }

    let source: SmallVec<Zst, 0> = std::iter::repeat_with(|| Zst).take(128).collect();
    let mut iter = source.into_iter();
    for _ in 0..100 {
        drop(iter.next());
    }
    for _ in 0..20 {
        drop(iter.next_back());
    }
    let result: SmallVec<Zst, 8> = iter.collect();
    assert_eq!(result.len(), 8);
    assert!(!result.spilled());
    DROPS.with(|count| assert_eq!(count.get(), 120));
    drop(result);
    DROPS.with(|count| assert_eq!(count.get(), 128));
}

#[test]
fn collect_bounds_retained_capacity() {
    // Include front/back consumption, untouched preallocation, and the exact
    // half-capacity boundary for even and odd allocations.
    for (capacity, len, front, back, reuse) in [
        (4096, 4096, 4032, 0, false),
        (4096, 4096, 0, 4032, false),
        (4096, 4096, 2016, 2016, false),
        (4096, 64, 0, 0, false),
        (128, 128, 65, 0, false),
        (128, 128, 64, 0, true),
        (128, 128, 0, 64, true),
        (129, 129, 65, 0, false),
        (129, 129, 64, 0, true),
        (129, 64, 0, 0, false),
        (129, 65, 0, 0, true),
        (4096, 4096, 4088, 0, false),
        (4096, 4096, 0, 4096, false)
    ] {
        let mut source: SmallVec<usize, 8> = SmallVec::with_capacity(capacity);
        source.extend(0..len);
        assert_eq!(source.capacity(), capacity);
        let source_ptr = source.as_ptr();
        let mut iter = source.into_iter();
        for _ in 0..front {
            iter.next();
        }
        for _ in 0..back {
            iter.next_back();
        }
        let result: SmallVec<usize, 8> = iter.collect();
        assert_eq!(&result[..], &(front..len - back).collect::<Vec<_>>());
        assert_eq!(result.spilled(), result.len() > 8);
        if cfg!(feature = "specialization") && reuse {
            assert_eq!(result.as_ptr(), source_ptr);
            assert_eq!(result.capacity(), capacity);
        } else {
            assert!(result.capacity() <= 8.max(2 * result.len()));
            if cfg!(feature = "specialization") && result.spilled() {
                assert_eq!(result.capacity(), result.len());
                assert_ne!(result.as_ptr(), source_ptr);
            }
        }
    }
}

struct Tracked(Rc<Cell<usize>>);

impl Drop for Tracked {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn collect_drops_every_element_once() {
    for (capacity, len, front, back) in [
        (10, 10, 0, 0),
        (10, 10, 3, 2),
        (10, 10, 8, 0),
        (10, 10, 10, 0),
        (128, 128, 119, 0),
        (128, 128, 0, 119),
        (128, 128, 60, 59),
        (128, 9, 0, 0)
    ] {
        let drops: Vec<_> = (0..len).map(|_| Rc::new(Cell::new(0))).collect();
        let mut source: SmallVec<Tracked, 4> = SmallVec::with_capacity(capacity);
        source.extend(drops.iter().cloned().map(Tracked));
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
                usize::from(index < front || index >= len - back)
            );
        }
        drop(result);
        assert!(drops.iter().all(|count| count.get() == 1));
    }
}
