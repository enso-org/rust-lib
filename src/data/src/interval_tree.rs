//! Specialized, high-performance interval tree implementation.

use crate::prelude::*;
use std::cmp::Ordering;




// ================
// === Interval ===
// ================

/// Closed interval. For example, [`Interval(1,2)`] means `[1,2]` in math.
#[derive(Debug,Clone,Copy,Default,Eq,PartialEq)]
#[allow(missing_docs)]
pub struct Interval {
    pub start : usize,
    pub end   : usize,
}

/// Constructor.
#[allow(non_snake_case)]
pub fn Interval(start:usize, end:usize) -> Interval {
    Interval {start,end}
}

impl From<Interval> for RightOpenInterval {
    fn from(t:Interval) -> Self {
        let start = t.start;
        let end   = t.end.saturating_add(1);
        Self {start,end}
    }
}

impl From<RightOpenInterval> for Interval {
    fn from(t:RightOpenInterval) -> Self {
        let start = t.start;
        let end   = t.end.saturating_sub(1);
        Self {start,end}
    }
}



// =========================
// === RightOpenInterval ===
// =========================

/// Right side opened interval. For example, [`RightOpenInterval(1,2)`] means `[1,2[` in math.
#[derive(Debug,Clone,Copy,Default,Eq,PartialEq)]
#[allow(missing_docs)]
pub struct RightOpenInterval {
    pub start : usize,
    pub end   : usize,
}

/// Constructor.
#[allow(non_snake_case)]
pub fn RightOpenInterval(start:usize, end:usize) -> RightOpenInterval {
    RightOpenInterval {start,end}
}

impl RightOpenInterval {
    /// Compare the value to this interval. In case the value will be "close" to the right side
    /// of the interval, it will be considered to be included in. For example, for the
    /// [`RightOpenInterval(1,2)`], the value `2` is considered to be "close".
    ///
    /// This allows for performant insertion of intervals into the [`IntervalTree`] (not implemented
    /// yet).
    pub fn cmp_close_to_value(&self, value:usize) -> Ordering {
        if      self.start > value { Ordering::Greater }
        else if self.end   < value { Ordering::Less }
        else                       { Ordering::Equal }
    }

    /// Checks whether the `end` value is bigger than `start` value.
    pub fn check_valid(&self) -> bool {
        self.start < self.end
    }
}

impl From<&RightOpenInterval> for RightOpenInterval {
    fn from(t:&RightOpenInterval) -> RightOpenInterval {
        *t
    }
}



// ====================
// === IntervalTree ===
// ====================

/// High performance interval tree implementation. It is highly specialized, but could be
/// generalized in the future without any performance loss. It allows inserting new values and it
/// automatically merges intervals that are next to each other. All intervals stored in the tree are
/// sorted and non-overlapping.
#[derive(Debug,Clone,Default,Eq,PartialEq)]
pub struct IntervalTree {
    /// The internal representation uses [`RightOpenInterval`] because it allows for the most
    /// performant implementation of insertion of new intervals (although it is not implemented in
    /// this library yet). Inserting another [`RightOpenInterval`] requires the smallest number of
    /// `saturating_add` and `saturating_sub`, and only two binary searches.
    vec        : SmallVec<[RightOpenInterval;256]>,
    item_count : usize,
}

impl IntervalTree {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }

    /// The number of items in this tree.
    pub fn item_count(&self) -> usize {
        self.item_count
    }

    /// The number of intervals in this tree.
    pub fn interval_count(&self) -> usize {
        self.vec.len()
    }

    /// Get the interval by index.
    pub fn index(&self, ix:usize) -> Option<RightOpenInterval> {
        (ix < self.interval_count()).as_some_from(||self.vec[ix])
    }
}

impl IntervalTree {
    /// Insert a new element. In case the element will be next to an interval, or between them,
    /// it will be merged with them.
    pub fn insert(&mut self, value:usize) {
        let index = self.vec.binary_search_by(|p|p.cmp_close_to_value(value));
        self.item_count += 1;
        match index {
            Err(index) => {
                if self.vec.len() > index && self.vec[index].start == value + 1 {
                    self.vec[index].start -= 1;
                } else {
                    self.vec.insert(index,Interval(value,value).into())
                }
            }
            Ok(index) => {
                if self.vec[index].end == value {
                    let next_index = index + 1;
                    if self.vec.len() > next_index && self.vec[next_index].start == value + 1 {
                        self.vec[index].end = self.vec[next_index].end;
                        self.vec.remove(next_index);
                    } else {
                        self.vec[index].end += 1
                    }
                }
                // Fully contained.
                else {
                    self.item_count -= 1;
                }
            }
        }
    }

    /// Take the first item and shrink or remove the first interval.
    pub fn take_first_item(&mut self) -> Option<usize> {
        let len = self.vec.len();
        let (out,truncate) = if len == 0 {
            (None,false)
        } else {
            let first_interval    = &mut self.vec[0];
            let out               = first_interval.start;
            first_interval.start += 1;
            self.item_count      -= 1;
            (Some(out),!first_interval.check_valid())
        };
        if truncate {
            self.vec.remove(0);
        }
        out
    }

    /// Take the last item and shrink or remove the last interval.
    pub fn take_last_item(&mut self) -> Option<usize> {
        let len = self.vec.len();
        let (out,truncate) = if len == 0 {
            (None,false)
        } else {
            let last_index     = len - 1;
            let last_interval  = &mut self.vec[last_index];
            last_interval.end -= 1;
            self.item_count   -= 1;
            (Some(last_interval.end),!last_interval.check_valid())
        };
        if truncate {
            self.vec.truncate(len - 1);
        }
        out
    }

    /// The first interval of the tree.
    pub fn first_interval(&mut self) -> Option<Interval> {
        self.vec.first().map(|t|(*t).into())
    }

    /// The last interval of the tree.
    pub fn last_interval(&mut self) -> Option<Interval> {
        self.vec.last().map(|t|(*t).into())
    }

    /// The first item of the first interval.
    pub fn first_item(&mut self) -> Option<usize> {
        self.first_interval().map(|t|t.start)
    }

    /// The last item of the last interval.
    pub fn last_item(&mut self) -> Option<usize> {
        self.last_interval().map(|t|t.end)
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(vals:&[(usize,usize)]) -> SmallVec::<[RightOpenInterval;4]> {
        SmallVec::from_vec(vals.iter().map(|t|Interval(t.0,t.1).into()).collect_vec())
    }

    #[test]
    fn test_1() {
        let mut v = IntervalTree::new();
        v.insert(10) ; assert_eq!(v.vec,raw(&[(10,10)]));
        v.insert(9)  ; assert_eq!(v.vec,raw(&[(9,10)]));
        v.insert(9)  ; assert_eq!(v.vec,raw(&[(9,10)]));
        v.insert(10) ; assert_eq!(v.vec,raw(&[(9,10)]));
        v.insert(11) ; assert_eq!(v.vec,raw(&[(9,11)]));
        v.insert(7)  ; assert_eq!(v.vec,raw(&[(7,7),(9,11)]));
        v.insert(7)  ; assert_eq!(v.vec,raw(&[(7,7),(9,11)]));
        v.insert(6)  ; assert_eq!(v.vec,raw(&[(6,7),(9,11)]));
        v.insert(8)  ; assert_eq!(v.vec,raw(&[(6,11)]));
        v.insert(13) ; assert_eq!(v.vec,raw(&[(6,11),(13,13)]));
        v.insert(12) ; assert_eq!(v.vec,raw(&[(6,13)]));
        v.insert(15) ; assert_eq!(v.vec,raw(&[(6,13),(15,15)]));
        v.insert(16) ; assert_eq!(v.vec,raw(&[(6,13),(15,16)]));
        assert_eq!(v.first_item()     , Some(6));
        assert_eq!(v.last_item()      , Some(16));
        assert_eq!(v.interval_count() , 2);
        assert_eq!(v.item_count()     , 10);
        assert_eq!(v.take_last_item()  , Some(16)) ; assert_eq!(v.vec,raw(&[(6,13),(15,15)]));
        assert_eq!(v.take_first_item() , Some(6))  ; assert_eq!(v.vec,raw(&[(7,13),(15,15)]));
        assert_eq!(v.take_last_item()  , Some(15)) ; assert_eq!(v.vec,raw(&[(7,13)]));
        assert_eq!(v.take_first_item() , Some(7))  ; assert_eq!(v.vec,raw(&[(8,13)]));
        assert_eq!(v.take_last_item()  , Some(13)) ; assert_eq!(v.vec,raw(&[(8,12)]));
        assert_eq!(v.take_first_item() , Some(8))  ; assert_eq!(v.vec,raw(&[(9,12)]));
        assert_eq!(v.take_last_item()  , Some(12)) ; assert_eq!(v.vec,raw(&[(9,11)]));
        assert_eq!(v.take_first_item() , Some(9))  ; assert_eq!(v.vec,raw(&[(10,11)]));
        assert_eq!(v.take_last_item()  , Some(11)) ; assert_eq!(v.vec,raw(&[(10,10)]));
        assert_eq!(v.take_first_item() , Some(10)) ; assert_eq!(v.vec,raw(&[]));
        v.insert(10) ; assert_eq!(v.vec,raw(&[(10,10)]));
        assert_eq!(v.take_last_item()  , Some(10)) ; assert_eq!(v.vec,raw(&[]));
        assert_eq!(v.take_first_item() , None)     ; assert_eq!(v.vec,raw(&[]));
        assert_eq!(v.take_last_item()  , None)     ; assert_eq!(v.vec,raw(&[]));
        assert_eq!(v.interval_count() , 0);
        assert_eq!(v.item_count()     , 0);
        assert_eq!(v.first_item()     , None);
        assert_eq!(v.last_item()      , None);
    }

    #[test]
    fn test_2() {
        let mut v = IntervalTree::new();
        v.insert(10) ; assert_eq!(v.vec,raw(&[(10,10)]));
        v.insert(12) ; assert_eq!(v.vec,raw(&[(10,10),(12,12)]));
        v.insert(14) ; assert_eq!(v.vec,raw(&[(10,10),(12,12),(14,14)]));
        v.insert(13) ; assert_eq!(v.vec,raw(&[(10,10),(12,14)]));
        v.insert(11) ; assert_eq!(v.vec,raw(&[(10,14)]));
    }
}


extern crate test;

#[cfg(test)]
mod benches {
    use super::*;
    use test::Bencher;

    /// # Results
    /// 10^4 -> 0.18 ms
    /// 10^5 -> 2.3 ms
    /// 10^6 -> 30 ms
    /// 10^7 -> 420 ms
    /// 10^8 -> 2.2 s
    #[bench]
    fn bench_insert_ascending(b:&mut Bencher) {
        b.iter(|| {
            let mut v = IntervalTree::new();
            for i in 0 .. 1000_000_00 {
                v.insert(i*2);
            }
        });
    }
}
