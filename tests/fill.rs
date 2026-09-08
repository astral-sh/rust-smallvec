use {
    smallvec::{
        SmallVec,
        from_elem
    },
    std::{
        cell::Cell,
        panic::AssertUnwindSafe,
        rc::Rc
    }
};

#[test]
fn from_elem_boundaries() {
    for n in [0, 1, 7, 8, 9, 64] {
        for value in [0, 19, 255] {
            let v: SmallVec<u8, 8> = from_elem(value, n);
            assert_eq!(&v[..], vec![value; n]);
            assert_eq!(v.spilled(), n > 8);
        }
        let v: SmallVec<(), 8> = from_elem((), n);
        assert_eq!(v.len(), n);
        assert!(!v.spilled());
    }
}

struct Tracked {
    clones: Rc<Cell<usize>>,
    drops: Rc<Cell<usize>>,
    panic_at: usize
}

impl Clone for Tracked {
    fn clone(&self) -> Self {
        let clones = self.clones.get() + 1;
        self.clones.set(clones);
        assert_ne!(clones, self.panic_at, "clone panic");
        Self {
            clones: self.clones.clone(),
            drops: self.drops.clone(),
            panic_at: self.panic_at
        }
    }
}

impl Drop for Tracked {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

#[test]
fn from_elem_clones_and_drops() {
    for n in [0usize, 1, 8, 9] {
        let clones = Rc::new(Cell::new(0));
        let drops = Rc::new(Cell::new(0));
        let value = Tracked {
            clones: clones.clone(),
            drops: drops.clone(),
            panic_at: usize::MAX
        };
        let v: SmallVec<Tracked, 8> = from_elem(value, n);
        assert_eq!(clones.get(), n.saturating_sub(1));
        drop(v);
        assert_eq!(drops.get(), n.max(1));
    }
}

#[test]
fn from_elem_clone_panic_drops_initialized_values() {
    for n in [8, 9] {
        let clones = Rc::new(Cell::new(0));
        let drops = Rc::new(Cell::new(0));
        let value = Tracked {
            clones: clones.clone(),
            drops: drops.clone(),
            panic_at: 3
        };
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            let _: SmallVec<Tracked, 8> = from_elem(value, n);
        }));
        assert!(result.is_err());
        assert_eq!(clones.get(), 3);
        assert_eq!(drops.get(), 3);
    }
}
