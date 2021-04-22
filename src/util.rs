use std::{cmp::Ordering, ops::Range};

pub fn cmp_float(a: f32, b: f32) -> bool {
    (a - b).abs() <= f32::EPSILON * a.abs().max(b.abs())
}

pub fn cmp_range(v: usize, range: Range<usize>) -> Ordering {
    if v < range.start {
        Ordering::Less
    } else if v >= range.end {
        Ordering::Greater
    } else {
        Ordering::Equal
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
