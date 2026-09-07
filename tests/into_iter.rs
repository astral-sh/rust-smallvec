use {
    smallvec::SmallVec,
    std::{
        cell::{
            Cell,
            RefCell
        },
        iter::FromIterator,
        panic::{
            AssertUnwindSafe,
            catch_unwind
        },
        rc::Rc
    }
};

type Vector<T> = SmallVec<T, 16>;

#[test]
fn skip_matches_vec() {
    for &len in &[0, 1, 8, 16, 17, 32] {
        for &skip in &[0, 1, 3, len, len + 1, usize::MAX] {
            for &back in &[false, true] {
                for &partial in &[false, true] {
                    let mut actual = Vector::from_iter(0..len).into_iter();
                    let mut expected = Vec::from_iter(0..len).into_iter();
                    if partial {
                        assert_eq!(actual.next(), expected.next());
                        assert_eq!(actual.next_back(), expected.next_back());
                    }
                    let (a, e) = if back {
                        (actual.nth_back(skip), expected.nth_back(skip))
                    } else {
                        (actual.nth(skip), expected.nth(skip))
                    };
                    assert_eq!(a, e);
                    assert_eq!(actual.len(), expected.len());
                    assert_eq!(actual.size_hint(), expected.size_hint());
                    assert_eq!(actual.as_slice(), expected.as_slice());
                    assert_eq!(actual.collect::<Vec<_>>(), expected.collect::<Vec<_>>());
                }
            }
        }
    }
}

struct Tracked {
    id: usize,
    drops: Rc<RefCell<Vec<usize>>>,
    panic_on: Rc<Cell<Option<usize>>>
}

impl Drop for Tracked {
    fn drop(&mut self) {
        self.drops.borrow_mut().push(self.id);
        if self.panic_on.get() == Some(self.id) {
            self.panic_on.set(None);
            panic!("drop {}", self.id);
        }
    }
}

#[test]
fn skipped_items_are_dropped_once() {
    for &len in &[8, 16, 17] {
        for &back in &[false, true] {
            for &panic in &[false, true] {
                let drops = Rc::new(RefCell::new(Vec::new()));
                let panic_on = Rc::new(Cell::new(if panic {
                    Some(if back { len - 2 } else { 1 })
                } else {
                    None
                }));
                let mut iter = Vector::from_iter((0..len).map(|id| Tracked {
                    id,
                    drops: drops.clone(),
                    panic_on: panic_on.clone()
                }))
                .into_iter();
                let result = catch_unwind(AssertUnwindSafe(|| {
                    if back { iter.nth_back(3) } else { iter.nth(3) }
                }));
                assert_eq!(result.is_err(), panic);
                assert_eq!(drops.borrow().len(), 3);
                assert_eq!(iter.len(), len - if panic { 3 } else { 4 });
                drop(result);
                drop(iter);
                let mut actual = drops.borrow().clone();
                actual.sort();
                assert_eq!(actual, (0..len).collect::<Vec<_>>());
            }
        }
    }
}

thread_local! {
    static ZST_DROPS: Cell<usize> = Cell::new(0);
}

struct Zst;

impl Drop for Zst {
    fn drop(&mut self) {
        ZST_DROPS.with(|drops| drops.set(drops.get() + 1));
    }
}

#[test]
fn skipping_zero_sized_items() {
    for &back in &[false, true] {
        ZST_DROPS.with(|drops| drops.set(0));
        let mut iter = Vector::from_iter((0..32).map(|_| Zst)).into_iter();
        let item = if back { iter.nth_back(7) } else { iter.nth(7) };
        assert_eq!(iter.len(), 24);
        ZST_DROPS.with(|drops| assert_eq!(drops.get(), 7));
        drop(item);
        let item = if back {
            iter.nth_back(usize::MAX)
        } else {
            iter.nth(usize::MAX)
        };
        assert!(item.is_none());
        assert_eq!(iter.len(), 0);
        drop(iter);
        ZST_DROPS.with(|drops| assert_eq!(drops.get(), 32));
    }
}

#[test]
fn last_after_partial_consumption() {
    for len in [0, 1, 8, 16, 17] {
        let mut iter = Vector::from_iter(0..len).into_iter();
        let mut expected = Vec::from_iter(0..len).into_iter();
        assert_eq!(iter.next(), expected.next());
        assert_eq!(iter.next_back(), expected.next_back());
        assert_eq!(iter.last(), expected.last());
    }
}

#[test]
fn last_drops_remainder_when_a_destructor_panics() {
    for len in [8, 17] {
        let drops = Rc::new(RefCell::new(Vec::new()));
        let panic_on = Rc::new(Cell::new(Some(1)));
        let iter = Vector::from_iter((0..len).map(|id| Tracked {
            id,
            drops: drops.clone(),
            panic_on: panic_on.clone()
        }))
        .into_iter();
        assert!(catch_unwind(AssertUnwindSafe(|| iter.last())).is_err());
        let mut actual = drops.borrow().clone();
        actual.sort();
        assert_eq!(actual, (0..len).collect::<Vec<_>>());
    }
}

#[test]
fn skipping_large_zero_sized_length() {
    for &back in &[false, true] {
        for &skip in &[std::usize::MAX - 2, std::usize::MAX] {
            let mut vec = Vector::<()>::new();
            // All unit values are initialized, and ZST capacity is usize::MAX.
            unsafe {
                vec.set_len(std::usize::MAX - 1);
            }
            let mut iter = vec.into_iter();
            let item = if back {
                iter.nth_back(skip)
            } else {
                iter.nth(skip)
            };
            assert_eq!(
                item,
                if skip == std::usize::MAX {
                    None
                } else {
                    Some(())
                }
            );
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next_back(), None);
        }
    }
}
