//! Specialized, high-performance interval tree implementation.

use std::mem::MaybeUninit;



use crate::prelude::*;
use std::cmp::Ordering;


// ================
// === Interval ===
// ================

/// Closed interval. For example, [`Interval(1,2)`] means `[1,2]` in math.
#[derive(Clone,Copy,Default,Eq,PartialEq)]
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

impl Debug for Interval {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Interval({:?},{:?})", self.start, self.end)
    }
}

impl From<usize> for Interval {
    fn from(t:usize) -> Self {
        Interval(t,t)
    }
}

impl From<(usize,usize)> for Interval {
    fn from(t:(usize,usize)) -> Self {
        Interval(t.0,t.1)
    }
}

const DATA_SIZE : usize = 4;
type DataType = [Interval;4];
type DataTypeUninit = [MaybeUninit<Interval>;4];
const CHILDREN_SIZE : usize = 5;
type ChildrenType = [Tree;5];
type ChildrenTypeUninit = [MaybeUninit<Tree>;5];



// ============
// === Tree ===
// ============

#[derive(Clone)]
pub struct Tree {
    data_count : usize,
    data       : DataType,
    children   : Option<Box<ChildrenType>>
}

impl Default for Tree {
    #[allow(clippy::uninit_assumed_init)]
    fn default() -> Self {
        let data_count = 0;

        // src https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        let mut data: DataTypeUninit = unsafe { MaybeUninit::uninit().assume_init() };
        for elem in &mut data[..] { *elem = MaybeUninit::new(default()); }
        let data = unsafe { mem::transmute::<_,DataType>(data) };

        let children   = None;
        Self {data_count,data,children}
    }
}

impl PartialEq for Tree {
    fn eq(&self, other:&Self) -> bool {
        if self.data_count != other.data_count {
            return false;
        }
        for i in 0..self.data_count {
            if self.data[i] != other.data[i] {
                return false;
            }
        }
        match (&self.children,&other.children) {
            (None,None) => {}
            (Some(children1),Some(children2)) => {
                for i in 0..self.data_count+1 {
                    if children1[i] != children2[i] {
                        return false;
                    }
                }
            }
            _ => return false
        }
        true
    }
}

impl Eq for Tree {}

impl Debug for Tree {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut repr = vec![];
        if let Some(children) = &self.children {
            for i in 0..self.data_count {
                repr.push(format!("{:?}", children[i]));
                repr.push(format!("{:?}", self.data[i]));
            }
            repr.push(format!("{:?}", children[self.data_count]));
        } else {
            for i in 0..self.data_count {
                repr.push(format!("{:?}", self.data[i]));
            }
        }
        write!(f, "Tree({})", repr.join(","))
    }
}

impl Tree {
    fn empty_children_array() -> ChildrenType {
        // src https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        let mut children: ChildrenTypeUninit = unsafe { MaybeUninit::uninit().assume_init() };
        for elem in &mut children[..] { *elem = MaybeUninit::new(default()); }
        unsafe { mem::transmute::<_,ChildrenType>(children) }
    }

    fn unsafe_init_children(&mut self) -> &mut [Tree] {
        self.children = Some(Box::new(Self::empty_children_array()));
        self.children.as_mut().unwrap().deref_mut()
    }

    pub fn search(&self, t:usize) -> Result<usize,usize> {
        let mut out = Err(self.data_count);
        for i in 0..self.data_count {
            let interval = &self.data[i];
            if      t + 1 <  interval.start   { out = Err(i) ; break }
            else if t     <= interval.end + 1 { out = Ok(i)  ; break }
        }
        out
    }

    // fn search(&self, t:usize) -> Result<usize, usize>
    // {
    //     self.binary_search_by(|interval| {
    //         if      t + 1 <  interval.start   { Ordering::Less }
    //         else if t     <= interval.end + 1 { Ordering::Equal }
    //         else { Ordering::Greater }
    //     })
    // }

    #[inline]
    fn binary_search_by<'a, F>(&'a self, mut f: F) -> Result<usize, usize>
        where
            F: FnMut(&'a Interval) -> Ordering,
    {
        let s = &self.data;
        let mut size = s.len();
        if size == 0 {
            return Err(0);
        }
        let mut base = 0usize;
        while size > 1 {
            let half = size / 2;
            let mid = base + half;
            // SAFETY: the call is made safe by the following inconstants:
            // - `mid >= 0`: by definition
            // - `mid < size`: `mid = size / 2 + size / 4 + size / 8 ...`
            let cmp = f(unsafe { s.get_unchecked(mid) });
            base = if cmp == std::cmp::Ordering::Greater { base } else { mid };
            size -= half;
        }
        // SAFETY: base is always in [0, size) because base <= mid.
        let cmp = f(unsafe { s.get_unchecked(base) });
        if cmp == std::cmp::Ordering::Equal { Ok(base) } else { Err(base + (cmp == std::cmp::Ordering::Less) as usize) }
    }

    fn split_leaves(&self, left_split_index:usize, right_split_index:usize) -> (Tree,Tree) {
        let mut left = Tree::default();
        left.data_count = left_split_index;
        left.data[0..left_split_index].copy_from_slice(&self.data[0..left_split_index]);

        let mut right = Tree::default();
        right.data_count = DATA_SIZE - right_split_index;
        right.data[0..right.data_count].copy_from_slice(&self.data[right_split_index..]);

        (left,right)
    }

    fn split
    (data:&mut DataType, children:&mut ChildrenType, left:Tree, right:Tree, split_index:usize)
    -> (Tree,Tree) {
        let mut p_left = Tree::default();
        p_left.data_count = split_index;
        p_left.data[0..split_index].copy_from_slice(&data[0..split_index]);

        let mut left_children = Self::empty_children_array();
        left_children[0..split_index].clone_from_slice(&children[0..split_index]);
        left_children[split_index] = left;
        p_left.children = Some(Box::new(left_children));

        let mut p_right = Tree::default();
        p_right.data_count = DATA_SIZE - split_index;
        p_right.data[0..p_right.data_count].copy_from_slice(&data[split_index..]);

        let mut right_children = Self::empty_children_array();
        right_children[1..p_right.data_count+1].clone_from_slice(&children[split_index+1..]);
        right_children[0] = right;
        p_right.children = Some(Box::new(right_children));

        (p_left,p_right)
    }

    pub fn insert(&mut self, t:usize) {
        println!("--- insert");

        if let Some((median,left,right)) = self.insert_internal(t) {
            let mut new_root = Tree::default();
            new_root.data_count = 1;
            new_root.data[0] = median;
            let new_root_children = new_root.unsafe_init_children();
            new_root_children[0] = left;
            new_root_children[1] = right;
            *self = new_root;
        }
    }

    pub fn insert_internal(&mut self, t:usize) -> Option<(Interval,Tree,Tree)> {
        println!("--- insert_internal");

        let pos = self.search(t);

        println!("POS: {:?}",pos);

        match self.search(t) {
            Err(pos) => {
                match &mut self.children {
                    None => {
                        println!("--- 0");

                        if self.data_count < DATA_SIZE {
                            println!("--- 1");
                            // Insert Case (1)
                            self.data[pos..].rotate_right(1);
                            self.data[pos] = Interval(t,t);
                            self.data_count += 1;
                            None
                        } else {
                            let median_index = DATA_SIZE / 2;
                            let (median,(left,right)) = if (pos == median_index) {
                                // Insert Case (2)
                                (Interval(t,t),self.split_leaves(median_index,median_index))
                            } else if (pos < median_index) {
                                // Insert Case (3)
                                let (mut left,right) = self.split_leaves(median_index-1, median_index);
                                left.insert_internal(t);
                                (self.data[median_index-1],(left,right))
                            } else {
                                // Insert Case (4)
                                let (left, mut right) = self.split_leaves(median_index, median_index+1);
                                right.insert_internal(t);
                                (self.data[median_index],(left,right))
                            };
                            Some((median,left,right))
                        }
                    }
                    Some(children) => {
                        if let Some((median,left,right)) = children[pos].insert_internal(t) {
                            if self.data_count < DATA_SIZE {
                                // Insert Case (1-4)
                                self.data[pos..].rotate_right(1);
                                children[pos..].rotate_right(1);
                                self.data[pos] = median;
                                children[pos] = left;
                                children[pos+1] = right;
                                self.data_count += 1;
                                None
                            } else {

                                //zakomentowanie tego brancha usuwa stackoverflow, mimmo ze tu nigdy niewchodzimy

                                let median_index = DATA_SIZE / 2;
                                let data  = &mut self.data;

                                if (pos == median_index) {
                                    // Insert Case (5)
                                    let split = |ix| Self::split(data,children,left,right,ix);

                                    let (p_left,p_right) = split(median_index);
                                    Some((median,p_left,p_right))
                                } else if (pos < median_index) {
                                    // Insert Case (6)
                                    let split = |ix| Self::split(data,children,left,right,ix);

                                    let (p_left,p_right) = split(median_index-1);
                                    self.data[pos] = median;
                                    Some((self.data[median_index-1],p_left,p_right))
                                } else {
                                    // Insert Case (7)
                                    // let (p_left,p_right) = split(median_index+1);
                                    //self.data[pos] = median;

                                    // t!(t!(10), 20, t!(30), 40, t!(50), 60, t!(70), 80, t!(90,92,94,96))


                                    // Tree(
                                    //     Tree(
                                    //         Interval(96,96),
                                    //         Interval(98,98)
                                    //     ),
                                    //     Interval(80,80),
                                    //     Tree(
                                    //         Interval(90,90),
                                    //         Interval(92,92),
                                    //         Interval(94,94),
                                    //         Interval(96,96)
                                    //     )
                                    // )

                                    let split_index = median_index;

                                    let mut p_left = Tree::default();
                                    p_left.data_count = split_index;
                                    p_left.data[0..split_index].copy_from_slice(&data[0..split_index]);
                                    let mut left_children = Self::empty_children_array();
                                    left_children[0..split_index+1].clone_from_slice(&children[0..split_index+1]);
                                    // left_children[split_index] = Box::new(left);
                                    p_left.children = Some(Box::new(left_children));



                                    let right_split_index = median_index + 1;
                                    let mut p_right = Tree::default();
                                    p_right.data_count = DATA_SIZE - right_split_index;
                                    // println!("p_right.data_count: {:?}",p_right.data_count);

                                    p_right.data[0..p_right.data_count].copy_from_slice(&data[right_split_index..]);
                                    let mut right_children = Self::empty_children_array();
                                    right_children[0..p_right.data_count+1].clone_from_slice(&children[right_split_index..]);

                                    let iii = pos-right_split_index;
                                    right_children[iii..].rotate_right(1);
                                    right_children[iii] = left;
                                    right_children[iii+1] = right;
                                    p_right.data[iii..].rotate_right(1);
                                    p_right.data[iii] = median;
                                    p_right.data_count += 1;

                                    // right_children[0] = Box::new(right);
                                    p_right.children = Some(Box::new(right_children));

                                    // println!("iii: {:?}",iii);
                                    // println!("pos: {:?}",pos);
                                    // println!("sub-median: {:?}",median);
                                    // println!("median: {:?}",self.data[median_index]);
                                    // println!("p_left: {:?}",p_left);
                                    // println!("p_right: {:?}",p_right);
                                    Some((self.data[split_index],p_left,p_right))
                                }
                            }
                        } else { None }
                    },
                }
            },
            Ok(pos) => {
                let interval = &mut self.data[pos];
                if t < interval.start {
                    interval.start = t;
                }
                else if t > interval.end {
                    interval.end = t;
                    let next_pos = pos + 1;
                    if next_pos < self.data_count {
                        let next_interval = self.data[next_pos];
                        if next_interval.start == t + 1 {
                            // Merging intervals.
                            let interval = &mut self.data[pos];
                            interval.end = next_interval.end;
                            self.data[next_pos..].rotate_left(1);
                            self.data_count -= 1;
                        }
                    }
                }
                None
            }
        }
    }


    pub fn delete(&mut self, t:usize) -> bool {
        match self.search(t) {
            Ok(pos) => {
                match &mut self.children {
                    None => {
                        // Delete Case (1)
                        self.data[pos..].rotate_left(1);
                        self.data_count -= 1;
                        false
                    },
                    Some(children) => todo!()
                }
            }
            Err(pos) => {
                match &mut self.children {
                    None => {
                        // Delete Case (X)
                        false
                    },
                    Some(children) => {
                        if children[pos].delete(t) {
                            todo!()
                        } else {
                            false
                        }
                    }
                }
            }
        }
    }

    fn unsafe_take_smallest_no_rebalance(&mut self) -> (Interval,bool) {
        if let Some(children) = &mut self.children {
            children[0].unsafe_take_smallest_no_rebalance()
        } else {
            let out = self.data[0];
            self.data[..].rotate_left(1);
            self.data_count -= 1;
            let needs_rebalance = self.data_count == 0;
            (out,needs_rebalance)
        }
    }

    fn unsafe_take_greatest_no_rebalance(&mut self) -> (Interval,bool) {
        if let Some(children) = &mut self.children {
            children[self.data_count].unsafe_take_greatest_no_rebalance()
        } else {
            let out = self.data[self.data_count-1];
            self.data_count -= 1;
            let needs_rebalance = self.data_count == 0;
            (out,needs_rebalance)
        }
    }

    pub fn to_vec(&self) -> Vec<Interval> {
        let mut v = vec![];
        if let Some(children) = &self.children {
            for i in 0..self.data_count {
                v.extend(children[i].to_vec());
                v.push(self.data[i])
            }
            v.extend(children[self.data_count].to_vec());
        } else {
            for i in 0..self.data_count {
                v.push(self.data[i])
            }
        }
        v
    }
}

trait FromSorted<T> {
    fn from_sorted(t:T) -> Self;
}

trait LeafFromSorted<T> {
    fn leaf_from_sorted(t:T) -> Self;
}

impl FromSorted<(Tree,(usize,usize),Tree)> for Tree {
    fn from_sorted(t:(Tree,(usize,usize),Tree)) -> Self {
        let mut tree = Tree::default();
        tree.data_count = 1;
        tree.data[0] = Interval((t.1).0,(t.1).1);
        let mut children = Self::empty_children_array();
        children[0] = t.0;
        children[1] = t.2;
        tree.children = Some(Box::new(children));
        tree
    }
}

impl FromSorted<(Tree,usize,Tree)> for Tree {
    fn from_sorted(t:(Tree,usize,Tree)) -> Self {
        let mut tree = Tree::default();
        tree.data_count = 1;
        tree.data[0] = Interval(t.1,t.1);
        let mut children = Self::empty_children_array();
        children[0] = t.0;
        children[1] = t.2;
        tree.children = Some(Box::new(children));
        tree
    }
}

impl FromSorted<(Tree,usize,Tree,usize,Tree)> for Tree {
    fn from_sorted(t:(Tree,usize,Tree,usize,Tree)) -> Self {
        let mut tree = Tree::default();
        tree.data_count = 2;
        tree.data[0] = Interval(t.1,t.1);
        tree.data[1] = Interval(t.3,t.3);
        let mut children = Self::empty_children_array();
        children[0] = t.0;
        children[1] = t.2;
        children[2] = t.4;
        tree.children = Some(Box::new(children));
        tree
    }
}

impl FromSorted<(Tree,usize,Tree,usize,Tree,usize,Tree)> for Tree {
    fn from_sorted(t:(Tree,usize,Tree,usize,Tree,usize,Tree)) -> Self {
        let mut tree = Tree::default();
        tree.data_count = 3;
        tree.data[0] = Interval(t.1,t.1);
        tree.data[1] = Interval(t.3,t.3);
        tree.data[2] = Interval(t.5,t.5);
        let mut children = Self::empty_children_array();
        children[0] = t.0;
        children[1] = t.2;
        children[2] = t.4;
        children[3] = t.6;
        tree.children = Some(Box::new(children));
        tree
    }
}

impl FromSorted<(Tree,usize,Tree,usize,Tree,usize,Tree,usize,Tree)> for Tree {
    fn from_sorted(t:(Tree,usize,Tree,usize,Tree,usize,Tree,usize,Tree)) -> Self {
        let mut tree = Tree::default();
        tree.data_count = 4;
        tree.data[0] = Interval(t.1,t.1);
        tree.data[1] = Interval(t.3,t.3);
        tree.data[2] = Interval(t.5,t.5);
        tree.data[3] = Interval(t.7,t.7);
        let mut children = Self::empty_children_array();
        children[0] = t.0;
        children[1] = t.2;
        children[2] = t.4;
        children[3] = t.6;
        children[4] = t.8;
        tree.children = Some(Box::new(children));
        tree
    }
}


impl<T1> LeafFromSorted<(T1,)> for Tree
    where T1:Into<Interval> {
    fn leaf_from_sorted(t:(T1,)) -> Self {
        let mut tree = Tree::default();
        tree.data_count = 1;
        tree.data[0] = t.0.into();
        tree
    }
}

impl<T1,T2> LeafFromSorted<(T1,T2)> for Tree
    where T1:Into<Interval>, T2:Into<Interval> {
    fn leaf_from_sorted(t:(T1,T2)) -> Self {
        let mut tree = Tree::default();
        tree.data_count = 2;
        tree.data[0] = t.0.into();
        tree.data[1] = t.1.into();
        tree
    }
}

impl<T1,T2,T3> LeafFromSorted<(T1,T2,T3)> for Tree
    where T1:Into<Interval>, T2:Into<Interval>, T3:Into<Interval> {
    fn leaf_from_sorted(t:(T1,T2,T3)) -> Self {
        let mut tree = Tree::default();
        tree.data_count = 3;
        tree.data[0] = t.0.into();
        tree.data[1] = t.1.into();
        tree.data[2] = t.2.into();
        tree
    }
}

impl<T1,T2,T3,T4> LeafFromSorted<(T1,T2,T3,T4)> for Tree
    where T1:Into<Interval>, T2:Into<Interval>, T3:Into<Interval>, T4:Into<Interval> {
    fn leaf_from_sorted(t:(T1,T2,T3,T4)) -> Self {
        let mut tree = Tree::default();
        tree.data_count = 4;
        tree.data[0] = t.0.into();
        tree.data[1] = t.1.into();
        tree.data[2] = t.2.into();
        tree.data[3] = t.3.into();
        tree
    }
}

impl FromSorted<((usize,usize),)> for Tree {
    fn from_sorted(t:((usize,usize),)) -> Self {
        let mut tree = Tree::default();
        tree.data_count = 1;
        tree.data[0] = Interval((t.0).0,(t.0).1);
        tree
    }
}



impl FromSorted<((usize,usize),(usize,usize))> for Tree {
    fn from_sorted(t:((usize,usize),(usize,usize))) -> Self {
        let mut tree = Tree::default();
        tree.data_count = 2;
        tree.data[0] = Interval((t.0).0,(t.0).1);
        tree.data[1] = Interval((t.1).0,(t.1).1);
        tree
    }
}

impl FromSorted<((usize,usize),(usize,usize),(usize,usize))> for Tree {
    fn from_sorted(t:((usize,usize),(usize,usize),(usize,usize))) -> Self {
        let mut tree = Tree::default();
        tree.data_count = 3;
        tree.data[0] = Interval((t.0).0,(t.0).1);
        tree.data[1] = Interval((t.1).0,(t.1).1);
        tree.data[2] = Interval((t.2).0,(t.2).1);
        tree
    }
}


impl FromSorted<((usize,usize),(usize,usize),(usize,usize),(usize,usize))> for Tree {
    fn from_sorted(t:((usize,usize),(usize,usize),(usize,usize),(usize,usize))) -> Self {
        let mut tree = Tree::default();
        tree.data_count = 4;
        tree.data[0] = Interval((t.0).0,(t.0).1);
        tree.data[1] = Interval((t.1).0,(t.1).1);
        tree.data[2] = Interval((t.2).0,(t.2).1);
        tree.data[3] = Interval((t.3).0,(t.3).1);
        tree
    }
}

impl FromSorted<(usize,)> for Tree {
    fn from_sorted(t:(usize,)) -> Self {
        Self::from_sorted(((t.0,t.0),))
    }
}

impl FromSorted<(usize,usize)> for Tree {
    fn from_sorted(t:(usize,usize)) -> Self {
        Self::from_sorted(((t.0,t.0),(t.1,t.1)))
    }
}

impl FromSorted<(usize,usize,usize)> for Tree {
    fn from_sorted(t:(usize,usize,usize)) -> Self {
        Self::from_sorted(((t.0,t.0),(t.1,t.1),(t.2,t.2)))
    }
}

impl FromSorted<(usize,usize,usize,usize)> for Tree {
    fn from_sorted(t:(usize,usize,usize,usize)) -> Self {
        Self::from_sorted(((t.0,t.0),(t.1,t.1),(t.2,t.2),(t.3,t.3)))
    }
}

fn t<T>(t:T) -> Tree
where Tree:FromSorted<T> {
    <Tree as FromSorted<T>>::from_sorted(t)
}

fn l<T>(t:T) -> Tree
where Tree:LeafFromSorted<T> {
    <Tree as LeafFromSorted<T>>::leaf_from_sorted(t)
}


macro_rules! t {
    ($($ts:tt)*) => {
        t(($($ts)*,))
    };
}

macro_rules! l {
    ($($ts:tt)*) => {
        l(($($ts)*,))
    };
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    // use std::os::unix::io::FromRawFd;
    // use std::fs::File;
    // #[test]
    // fn ttt() {
    //     let stdout = unsafe { File::from_raw_fd(1) };
    //     std::io::set_print(Some(Box::new(stdout)));
    //     // let mut v = Tree::default();
    //     // for i in 0 .. 100_0 {
    //     //     v.insert(i*2);
    //     // }
    //     // println!("{:?}",v);
    //     println!("hello1");
    //     let mut v = Tree::default();
    //     println!("hello2");
    //     v.insert(1);
    // }
    fn intervals(bounds:&[(usize,usize)]) -> Vec<Interval> {
        bounds.iter().copied().map(|(a,b)|Interval(a,b)).collect()
    }

    fn check(tree:&Tree, bounds:&[(usize,usize)]) {
        assert_eq!(tree.to_vec(),intervals(bounds));
    }

    #[test]
    fn leaf_insertion() {
        let mut v = Tree::default();
        check(&v,&[]);
        v.insert(1) ; check(&v,&[(1,1)]);
        v.insert(3) ; check(&v,&[(1,1),(3,3)]);
        v.insert(5) ; check(&v,&[(1,1),(3,3),(5,5)]);
        v.insert(6) ; check(&v,&[(1,1),(3,3),(5,6)]);
        v.insert(2) ; check(&v,&[(1,3),(5,6)]);
        v.insert(4) ; check(&v,&[(1,6)]);
    }

    #[test]
    fn deep_insertion() {
        let mut v = Tree::default();
        check(&v,&[]);
        v.insert(10)  ; check(&v,&[(10,10)]);
        v.insert(30)  ; check(&v,&[(10,10),(30,30)]);
        v.insert(50)  ; check(&v,&[(10,10),(30,30),(50,50)]);
        v.insert(70)  ; assert_eq!(v,t!(10,30,50,70));
        v.insert(90)  ; assert_eq!(v,t!( t!(10,30), 50, t!(70,90) ));
        v.insert(110) ; assert_eq!(v,t!( t!(10,30), 50, t!(70,90,110) ));
        v.insert(130) ; assert_eq!(v,t!( t!(10,30), 50, t!(70,90,110,130) ));
        v.insert(150) ; assert_eq!(v,t!( t!(10,30), 50, t!(70,90), 110, t!(130,150) ));
        v.insert(72)  ; assert_eq!(v,t!( t!(10,30), 50, t!(70,72,90), 110, t!(130,150) ));
        v.insert(74)  ; assert_eq!(v,t!( t!(10,30), 50, t!(70,72,74,90), 110, t!(130,150) ));
        v.insert(76)  ; assert_eq!(v,t!( t!(10,30), 50, t!(70,72), 74, t!(76,90), 110, t!(130,150) ));
        v.insert(32)  ; assert_eq!(v,t!( t!(10,30,32), 50, t!(70,72), 74, t!(76,90), 110, t!(130,150) ));
        v.insert(34)  ; assert_eq!(v,t!( t!(10,30,32,34), 50, t!(70,72), 74, t!(76,90), 110, t!(130,150) ));
        v.insert(36)  ; assert_eq!(v,t!( t!(10,30), 32, t!(34,36), 50, t!(70,72), 74, t!(76,90), 110, t!(130,150) ));
        v.insert(52)  ; assert_eq!(v,t!( t!(10,30), 32, t!(34,36), 50, t!(52,70,72), 74, t!(76,90), 110, t!(130,150) ));
        v.insert(54)  ; assert_eq!(v,t!( t!(10,30), 32, t!(34,36), 50, t!(52,54,70,72), 74, t!(76,90), 110, t!(130,150) ));
    }

    #[test]
    fn insert_case_1() {
        let mut v = t!(10,20) ; v.insert(0)  ; assert_eq!(v,t!(0,10,20));
        let mut v = t!(10,20) ; v.insert(15) ; assert_eq!(v,t!(10,15,20));
        let mut v = t!(10,20) ; v.insert(30) ; assert_eq!(v,t!(10,20,30));
    }

    #[test]
    fn insert_case_2() {
        let mut v1 = t!(t!(10,20,30,40),50,t!(60,70,80,90));
        let mut v2 = v1.clone();
        v1.insert(25) ; assert_eq!(v1,t!(t!(10,20),25,t!(30,40),50,t!(60,70,80,90)));
        v2.insert(75) ; assert_eq!(v2,t!(t!(10,20,30,40),50,t!(60,70),75,t!(80,90)));
    }

    #[test]
    fn insert_case_3() {
        let mut v1 = t!(t!(10,20,30,40),50,t!(60,70,80,90));
        let mut v2 = v1.clone();
        v1.insert(15) ; assert_eq!(v1,t!(t!(10,15),20,t!(30,40),50,t!(60,70,80,90)));
        v2.insert(0)  ; assert_eq!(v2,t!(t!(0,10) ,20,t!(30,40),50,t!(60,70,80,90)));
    }

    #[test]
    fn insert_case_4() {
        let mut v1 = t!(t!(10,20,30,40),50,t!(60,70,80,90));
        let mut v2 = v1.clone();
        v1.insert(35) ; assert_eq!(v1,t!(t!(10,20),30,t!(35,40),50,t!(60,70,80,90)));
        v2.insert(45) ; assert_eq!(v2,t!(t!(10,20),30,t!(40,45),50,t!(60,70,80,90)));
    }

    #[test]
    fn insert_case_5() {
        let mut v = t!(t!(10), 20, t!(30), 40, t!(50,52,54,56), 60, t!(70), 80, t!(90));
        v.insert(58);
        assert_eq!(v,t!(t!(t!(10),20,t!(30),40,t!(50,52)), 54, t!(t!(56,58),60,t!(70),80,t!(90))));
    }

    #[test]
    fn insert_case_6() {
        // let mut v = t!(t!(10,12,14,16), 20, t!(30), 40, t!(50), 60, t!(70), 80, t!(90));
        // v.insert(18);
        // assert_eq!(v,t!(t!(10,12), 14, t!(t!(16,18),20,t!(36),40,t!(50),60,t!(70),80,t!(90))));
        // Left:  t!(t!(t!(10,12,14,16),(20,20),t!((10,10),(12,12))),(40,40),t!(t!((16,16),(18,18)),(40,40),t!((50,50)),(60,60),t!((70,70)),(80,80),t!((90,90))))


        let mut v = t!(t!(10), 20, t!(30,32,34,36), 40, t!(50), 60, t!(70), 80, t!(90));
        v.insert(38);
        assert_eq!(v,t!(t!(t!(10),20,t!(30,32)), 34, t!(t!(36,38),40,t!(50),60,t!(70),80,t!(90))));
    }

    #[test]
    fn insert_case_7() {
        let mut v = t!(t!(10), 20, t!(30), 40, t!(50), 60, t!(70,72,74,76), 80, t!(90));
        v.insert(78);
        assert_eq!(v,
            t!(
                t!(
                    t!(10),
                    20,
                    t!(30),
                    40,
                    t!(50)
                ),
                60,
                t!(
                    t!(70,72),
                    74,
                    t!(76,78),
                    80,
                    t!(90)
                )
            )
        );

        let mut v = t!(t!(10), 20, t!(30), 40, t!(50), 60, t!(70), 80, t!(90,92,94,96));
        v.insert(98);
        assert_eq!(v,
            t!(
                t!(
                    t!(10),
                    20,
                    t!(30),
                    40,
                    t!(50)
                ),
                60,
                t!(
                    t!(70),
                    80,
                    t!(90,92),
                    94,
                    t!(96,98)
                )
            )
        );
    }

    #[test]
    fn insert_deep_bubble() {
        let mut v =
            t!(
                t!(100),
                200,
                t!(300),
                400,
                t!(
                    t!(500),
                    510,
                    t!(520),
                    530,
                    t!(540,542,544,546),
                    550,
                    t!(560),
                    570,
                    t!(580)
                ),
                600,
                t!(700),
                800,
                t!(900)
            );
        v.insert(548);
        assert_eq!(v,
            t!(
                t!(
                    t!(100),
                    200,
                    t!(300),
                    400,
                    t!(
                        t!(500),
                        510,
                        t!(520),
                        530,
                        t!(540,542)
                    )
                ),
                544,
                t!(
                    t!(
                        t!(546,548),
                        550,
                        t!(560),
                        570,
                        t!(580)
                    ),
                    600,
                    t!(700),
                    800,
                    t!(900)
                )
            )
        )
    }

    // #[test]
    // fn delete_case_1() {
    //     let mut v = l!((10,11),20,30) ; v.delete(11) ; assert_eq!(v,t!(10,20,30));
    //     let mut v = t!(10,20,30) ; v.delete(10) ; assert_eq!(v,t!(20,30));
    //     let mut v = t!(10,20,30) ; v.delete(20) ; assert_eq!(v,t!(10,30));
    //     let mut v = t!(10,20,30) ; v.delete(30) ; assert_eq!(v,t!(10,20));
    //     let mut v = t!(10,20)    ; v.delete(10) ; assert_eq!(v,t!(20));
    //     let mut v = t!(10,20)    ; v.delete(20) ; assert_eq!(v,t!(10));
    //     let mut v = t!(10)       ; v.delete(10) ; assert_eq!(v,t!::default());
    // }

    #[test]
    fn delete_case_X() {
        let mut v = t!(10,20,30) ; v.delete(0)  ; assert_eq!(v,t!(10,20,30));
        let mut v = t!(10,20,30) ; v.delete(15) ; assert_eq!(v,t!(10,20,30));
        let mut v = t!(10,20,30) ; v.delete(25) ; assert_eq!(v,t!(10,20,30));
        let mut v = t!(10,20,30) ; v.delete(35) ; assert_eq!(v,t!(10,20,30));
    }


    #[test]
    fn unsafe_take_smallest_no_rebalance() {
        let mut v = t!(10,20);
        assert_eq!(v.unsafe_take_smallest_no_rebalance(),(Interval(10,10),false));
        assert_eq!(v,t!(20));
        assert_eq!(v.unsafe_take_smallest_no_rebalance(),(Interval(20,20),true));
        assert_eq!(v,Tree::default());

        let mut v = t!(t!(10,12), 20, t!(30), 40, t!(50), 60, t!(70), 80, t!(90));
        assert_eq!(v.unsafe_take_smallest_no_rebalance(),(Interval(10,10),false));
        assert_eq!(v,t!(t!(12), 20, t!(30), 40, t!(50), 60, t!(70), 80, t!(90)));
        assert_eq!(v.unsafe_take_smallest_no_rebalance(),(Interval(12,12),true));
        assert_eq!(v,t!(Tree::default(), 20, t!(30), 40, t!(50), 60, t!(70), 80, t!(90)));
    }

    #[test]
    fn unsafe_take_greatest_no_rebalance() {
        let mut v = t!(10,20);
        assert_eq!(v.unsafe_take_greatest_no_rebalance(),(Interval(20,20),false));
        assert_eq!(v,t!(10));
        assert_eq!(v.unsafe_take_greatest_no_rebalance(),(Interval(10,10),true));
        assert_eq!(v,Tree::default());

        let mut v = t!(t!(10), 20, t!(30), 40, t!(50), 60, t!(70), 80, t!(90,92));
        assert_eq!(v.unsafe_take_greatest_no_rebalance(),(Interval(92,92),false));
        assert_eq!(v,t!(t!(10), 20, t!(30), 40, t!(50), 60, t!(70), 80, t!(90)));
        assert_eq!(v.unsafe_take_greatest_no_rebalance(),(Interval(90,90),true));
        assert_eq!(v,t!(t!(10), 20, t!(30), 40, t!(50), 60, t!(70), 80, Tree::default()));
    }
}


extern crate test;

#[cfg(test)]
mod benches {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_insert_ascending(b:&mut Bencher) {

        b.iter(|| {
            let mut v = Tree::default();
            v.insert(1);
            // for i in 0 .. 1000_0 {
            //     v.insert(i*2);
            // }
        });
    }
}

//
// Tree(
//     Tree(
//         Interval(0,0),
//         Interval(2,2)
//     ),
//     Interval(4,4),
//     Tree(
//         Interval(6,6),
//         Interval(8,8)
//     ),
//     Interval(10,10),
//     Tree(
//         Interval(12,12),
//         Interval(14,14)
//     ),
//     Interval(16,16),
//     Tree(
//         Interval(18,18),
//         Interval(20,20)
//     ),
//     Interval(22,22),
//     Tree(
//         Interval(24,24),
//         Interval(26,26),
//         Interval(28,28),
//         Interval(30,30)
//     )
// )

