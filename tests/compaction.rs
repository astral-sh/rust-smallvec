use {
    smallvec::SmallVec,
    std::{
        cell::Cell,
        panic::{
            AssertUnwindSafe,
            catch_unwind
        },
        rc::Rc
    }
};
type V<T> = SmallVec<T, 16>;

#[test]
fn compaction_matches_vec() {
    for len in [0, 1, 2, 15, 16, 17, 64].iter().copied() {
        for group in [1, 2, 3, 100].iter().copied() {
            let input: Vec<_> = (0..len).map(|x| (x / group, x)).collect();
            let mut expected = input.clone();
            let mut actual: V<_> = input.into_iter().collect();
            expected.dedup_by(|a, b| {
                b.1 += 1;
                a.0 == b.0
            });
            actual.dedup_by(|a, b| {
                b.1 += 1;
                a.0 == b.0
            });
            assert_eq!(actual.as_slice(), expected.as_slice());
        }
    }
}

struct Tracked {
    id: usize,
    drops: Rc<Vec<Cell<usize>>>,
    panic_at: Option<usize>
}
impl Drop for Tracked {
    fn drop(&mut self) {
        self.drops[self.id].set(self.drops[self.id].get() + 1);
        assert_ne!(self.panic_at, Some(self.id), "drop panic");
    }
}
fn tracked(len: usize, panic_at: Option<usize>) -> (Vec<Tracked>, Rc<Vec<Cell<usize>>>) {
    let drops = Rc::new((0..len).map(|_| Cell::new(0)).collect::<Vec<_>>());
    let values = (0..len)
        .map(|id| Tracked {
            id,
            drops: drops.clone(),
            panic_at
        })
        .collect();
    (values, drops)
}
fn ids(values: &[Tracked]) -> Vec<usize> {
    values.iter().map(|v| v.id).collect()
}

#[test]
fn dedup_predicate_panic_preserves_unprocessed_tail() {
    for len in [8, 32].iter().copied() {
        for panic_call in 0..len - 1 {
            let (input, drops) = tracked(len, None);
            let mut actual: V<_> = input.into_iter().collect();
            let mut calls = 0;
            assert!(
                catch_unwind(AssertUnwindSafe(|| actual.dedup_by(|a, b| {
                    let call = calls;
                    calls += 1;
                    assert_ne!(call, panic_call);
                    a.id / 2 == b.id / 2
                })))
                .is_err()
            );
            let read = panic_call + 1;
            let expected: Vec<_> = (0..read).step_by(2).chain(read..len).collect();
            assert_eq!(ids(&actual), expected);
            drop(actual);
            assert!(drops.iter().all(|x| x.get() == 1));
        }
    }
}

#[test]
fn dedup_destructor_panic_preserves_unprocessed_tail() {
    for len in [8, 32].iter().copied() {
        for panic_at in (1..len).step_by(2) {
            let (input, drops) = tracked(len, Some(panic_at));
            let mut actual: V<_> = input.into_iter().collect();
            assert!(
                catch_unwind(AssertUnwindSafe(
                    || actual.dedup_by(|a, b| a.id / 2 == b.id / 2)
                ))
                .is_err()
            );
            let expected: Vec<_> = (0..panic_at).step_by(2).chain(panic_at + 1..len).collect();
            assert_eq!(ids(&actual), expected);
            drop(actual);
            assert!(drops.iter().all(|x| x.get() == 1));
        }
    }
}

thread_local! { static ZST_DROPS: Cell<usize> = const { Cell::new(0) }; }
struct Zst;
impl Drop for Zst {
    fn drop(&mut self) {
        ZST_DROPS.with(|x| x.set(x.get() + 1));
    }
}
#[test]
fn dedup_zst_drops_once() {
    ZST_DROPS.with(|x| x.set(0));
    let mut values: V<_> = (0..32).map(|_| Zst).collect();
    values.dedup_by(|_, _| true);
    assert_eq!(values.len(), 1);
    ZST_DROPS.with(|x| assert_eq!(x.get(), 31));
    drop(values);
    ZST_DROPS.with(|x| assert_eq!(x.get(), 32));
}
