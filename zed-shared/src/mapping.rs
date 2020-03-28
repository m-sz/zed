use std::collections::HashMap;
use std::borrow::Borrow;
use std::hash::Hash;

pub struct Mapping<L, R> {
    lr: HashMap<L, R>,
    rl: HashMap<R, L>
}


impl<L: Hash + Eq + Clone, R: Hash + Eq + Clone> Mapping<L, R> {
    pub fn new() -> Self {
        Self {
            lr: HashMap::new(),
            rl: HashMap::new()
        }
    }

    pub fn by_left<'a, Q: ?Sized>(&'a self, k: &Q) -> Option<&R>
        where
            L: Borrow<Q> + 'a,
            Q: Hash + Eq
    {
        self.lr.get(k)
    }

    pub fn by_right<'a, Q: ?Sized>(&'a self, k: &Q) -> Option<&L>
        where
            R: Borrow<Q> + 'a,
            Q: Hash + Eq
    {
        self.rl.get(k)
    }

    pub fn insert(&mut self, left: L, right: R) {
        self.lr.insert(left.clone(), right.clone());
        self.rl.insert(right, left);
    }
}

