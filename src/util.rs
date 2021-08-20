use std::{cmp::Ordering, ops::Range};

#[cfg(test)]
mod test {
    #[test]
    fn cmp_range() {
        let v = vec![0..1, 1..2, 3..5, 5..8, 8..15];
        v.binary_search_by(|x| super::cmp_range(0, x.clone())).unwrap();
        v.binary_search_by(|x| super::cmp_range(1, x.clone())).unwrap();
        v.binary_search_by(|x| super::cmp_range(2, x.clone())).unwrap_err();
        v.binary_search_by(|x| super::cmp_range(5, x.clone())).unwrap();
        v.binary_search_by(|x| super::cmp_range(7, x.clone())).unwrap();
        v.binary_search_by(|x| super::cmp_range(8, x.clone())).unwrap();
        v.binary_search_by(|x| super::cmp_range(15, x.clone())).unwrap_err();
    }
}

pub fn cmp_float(a: f32, b: f32) -> bool {
    (a - b).abs() <= f32::EPSILON * a.abs().max(b.abs())
}

pub fn cmp_range(v: usize, range: Range<usize>) -> Ordering {
    if v < range.start {
        Ordering::Greater
    } else if v < range.end {
        Ordering::Equal
    } else {
        Ordering::Less
    }
}

pub struct WithPriority<P: Ord, Item> {
    priority: P,
    pub item: Item,
}
impl<P: Ord, Item> WithPriority<P, Item> {
    pub fn new(priority: P, item: Item) -> Self {
        Self { priority, item }
    }

    /// Get a reference to the with priority's priority.
    pub fn priority(&self) -> &P {
        &self.priority
    }
}
impl<P: Ord, Item> Eq for WithPriority<P, Item> {}
impl<P: Ord, Item> Ord for WithPriority<P, Item> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}
impl<P: Ord, Item> PartialOrd for WithPriority<P, Item> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<P: Ord, Item> PartialEq for WithPriority<P, Item> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}
