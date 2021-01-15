//! A dependency graph implementation optimized for sorting depth-indexed components.

use crate::prelude::*;

use std::collections::BTreeSet;



// ============
// === Node ===
// ============

/// A dependency graph node. Registers all incoming and outgoing edges.
#[derive(Clone,Debug,Default)]
#[allow(missing_docs)]
pub struct Node {
    pub ins : Vec<usize>,
    pub out : Vec<usize>,
}



// =======================
// === DependencyGraph ===
// =======================

/// Dependency graph keeping track of [`Node`]s and their dependencies.
///
/// The graph is not sparse, which means, that if there is a dependency to node of index `X`, the
/// graph will contain at least `X+1` nodes. Instead, you are allowed to topologically sort only a
/// subset of the graph.
///
/// This decision was dictated by the use cases of this structure. It is used to record dependencies
/// of display elements, however, in case you remove a display element, its depth-sorting
/// information should remain intact.
#[derive(Clone,Debug,Default)]
pub struct DependencyGraph {
    nodes : Vec<Node>
}

impl DependencyGraph {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }

    /// Insert a new dependency to the graph.
    pub fn insert_dependency(&mut self, first:usize, second:usize) {
        self.grow(1 + first.max(second));
        self.nodes[first].out.push(second);
        self.nodes[second].ins.push(first);
    }

    /// Removes all dependencies from nodes whose indexes do not belong to the provided slice.
    pub fn keep_only(&mut self, ixes:&[usize]) {
        self.unchecked_keep_only(&ixes.iter().copied().sorted().rev().collect_vec())
    }

    /// Removes all dependencies from nodes whose indexes do not belong to the provided slice.
    pub fn kept_only(mut self, ixes:&[usize]) -> Self {
        self.keep_only(ixes);
        self
    }

    /// Just like [`keep_only`], but the provided slice must be sorted in reversed order.
    pub fn unchecked_keep_only(&mut self, rev_sorted_ixes:&[usize]) {
        let mut keep         = rev_sorted_ixes.to_vec();
        let mut next_to_keep = keep.pop();
        for ix in 0..self.nodes.len() {
            if next_to_keep == Some(ix) {
                next_to_keep = keep.pop();
            } else {
                let node = mem::take(&mut self.nodes[ix]);
                for ix2 in node.ins { self.nodes[ix2].out.remove(ix); }
                for ix2 in node.out { self.nodes[ix2].ins.remove(ix); }
            }
        }
    }

    /// Just like [`kept_only`], but the provided slice must be sorted in reversed order.
    pub fn unchecked_kept_only(mut self, rev_sorted_ixes:&[usize]) -> Self {
        self.unchecked_keep_only(rev_sorted_ixes);
        self
    }

    /// Sorts the provided indexes in topological order based on the rules recorded in the graph.
    /// In case the graph is not a DAG, it will still be sorted by breaking cycles on elements with
    /// the smallest index.
    pub fn topo_sort(&self, ixes:&[usize]) -> Vec<usize> {
        self.unchecked_topo_sort(ixes.iter().copied().sorted().rev().collect_vec())
    }

    /// Just like [`topo_sort`], but the provided slice must be sorted in reversed order.
    pub fn unchecked_topo_sort(&self, rev_sorted_ixes:Vec<usize>) -> Vec<usize> {
        let mut sorted      = Vec::<usize>::new();
        let mut orphans     = BTreeSet::<usize>::new();
        let mut non_orphans = BTreeSet::<usize>::new();
        let mut this        = self.clone().unchecked_kept_only(&rev_sorted_ixes);
        let max_elem        = rev_sorted_ixes.first().copied().unwrap_or_default();
        sorted.reserve_exact(rev_sorted_ixes.len());
        this.grow(1 + max_elem);

        let mut nodes = this.nodes;
        for ix in rev_sorted_ixes.into_iter() {
            if nodes[ix].ins.is_empty() { orphans.insert(ix); }
            else                        { non_orphans.insert(ix); }
        }

        loop {
            match orphans.iter().next().copied() {
                None => {
                    match non_orphans.iter().next().copied() {
                        None => break,
                        Some(ix) => {
                            // NON DAG
                            non_orphans.remove(&ix);
                            orphans.insert(ix);
                        }
                    }
                },
                Some(ix) => {
                    sorted.push(ix);
                    orphans.remove(&ix);
                    for ix2 in mem::take(&mut nodes[ix].out) {
                        let ins = &mut nodes[ix2].ins;
                        ins.remove_item(&ix);
                        if ins.is_empty() && non_orphans.remove(&ix2) {
                            orphans.insert(ix2);
                        }
                    }
                }
            }
        }
        sorted
    }

    /// Grow the graph to the required size. This function will NOT shrink the graph if the provided
    /// size is smaller than the graph size.
    fn grow(&mut self, required_size:usize) {
        if required_size > self.nodes.len() {
            self.nodes.resize_with(required_size,default);
        }
    }
}



// ==============
// === Macros ===
// ==============

/// Utility macro allowing easy construction of the [`DependencyGraph`]. The following code:
/// ```
/// dependency_graph!(1->2, 2->3);
/// ```
/// will produce:
/// ```ignore
/// {
///     let mut graph = DependencyGraph::new();
///     graph.insert_dependency(1,2);
///     graph.insert_dependency(2,3);
///     graph
/// }
/// ```
#[macro_export]
macro_rules! dependency_graph {
    ($($fst:tt -> $snd:tt),* $(,)?) => {
        {
            #[allow(unused_mut)]
            let mut graph = DependencyGraph::new();
            $(graph.insert_dependency($fst,$snd);)*
            graph
        }
    };
}



// =============
// === Tests ===
// =============

extern crate test;

/// Asserts whether the graph will sort the provided slice in the same order as it was provided.
/// Please note, that the slice is sorted in order before being sorted topologically.
pub fn assert_valid_sort(graph:&DependencyGraph, sorted:&[usize]) {
    let sorted = sorted.to_vec();
    assert_eq!(graph.topo_sort(&sorted),sorted);
}

/// The same as [`assert_valid_sort`] but with a shorter syntax. Learn more about it by looking at
/// its usage below.
#[cfg(test)]
macro_rules! assert_valid_sort {
    ($($sorted:tt for {$($rules:tt)*})*) => {
        $({
            let graph = dependency_graph!{$($rules)*};
            assert_valid_sort(&graph,&$sorted);
        })*
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        assert_valid_sort!{
            []        for {}
            [0]       for {}
            [0,1]     for {}
            [0,1,2]   for {}
            [0,1,2,3] for {}
        }
    }

    #[test]
    fn test_non_overlapping_rules() {
        assert_valid_sort!{
            [1,0]     for {1->0}
            [0,2,1,3] for {2->1}
            [1,0,3,2] for {1->0,3->2}
        }
    }

    #[test]
    fn test_overlapping_rules() {
        assert_valid_sort!{
            [4,3,2,1,0]       for {4->3,3->2,2->1,1->0}
            [4,3,2,1,0]       for {3->2,4->3,1->0,2->1}
            [4,3,2,1,0]       for {1->0,2->1,3->2,4->3}
            [1,8,2,7,3,6,4,5] for {1->8,8->2,2->7,7->3,3->6,6->4,4->5}
        }
    }

    #[test]
    fn test_non_dag() {
        assert_valid_sort!{
            [0]     for {0->0}
            [1,2,0] for {0->0}
            [0,2,1] for {1->1}
            [0,1,2] for {2->2}
            [0,1]   for {0->1,1->0}
            [0,1,2] for {0->1,1->2,2->0}
            [0,1,2] for {0->0,0->1,0->2,1->0,1->1,1->2,2->0,2->1,2->2}
        }
    }
}

#[cfg(test)]
mod benches {
    use super::*;
    use test::Bencher;

    /// # Results (ms)
    ///
    ///   iters | time(ms) |
    ///   10^3  | 0.36     |
    ///   10^4  | 3.6      |
    ///   10^5  | 48.7     |
    #[bench]
    fn bench_ascending(b:&mut Bencher) {
        let iters     = 1_000;
        let out       = (0..iters).collect_vec();
        let mut graph = DependencyGraph::new();
        for (i,j) in out.iter().zip(out.iter().skip(1)) { graph.insert_dependency(*i,*j); }
        b.iter(move || assert_eq!(graph.topo_sort(&out),out));
    }

    /// # Results (ms)
    ///
    ///   iters | time(ms) |
    ///   10^3  | 0.37     |
    ///   10^4  | 4.0      |
    ///   10^5  | 55.2     |
    #[bench]
    fn bench_descending(b:&mut Bencher) {
        let iters     = 1_000;
        let out       = (0..iters).rev().collect_vec();
        let mut graph = DependencyGraph::new();
        for (i,j) in out.iter().zip(out.iter().skip(1)) { graph.insert_dependency(*i,*j); }
        b.iter(move || assert_eq!(graph.topo_sort(&out),out));
    }
}



// // =================================
// // === Sparse Dependency Sorting ===
// // =================================
//
// /// Sort the provided indexes slice according to the rules. Note, that in contrast to the
// /// [`DependencyGraph`], the rules are provided in a sparse fashion.
// #[allow(clippy::implicit_hasher)]
// pub fn sparse_depth_sort
// (indexes:&[usize], elem_above_elems:&HashMap<usize,Vec<usize>>) -> Vec<usize> {
//
//     // === Remove from `elem_above_elems` all indexes which are not present in `indexes` ===
//
//     let ids_set = HashSet::<usize>::from_iter(indexes.iter().copied());
//     let mut elem_above_elems = elem_above_elems.clone();
//     let mut missing = vec![];
//     for (elem,above_elems) in &mut elem_above_elems {
//         above_elems.retain(|id| ids_set.contains(id));
//         if above_elems.is_empty() {
//             missing.push(*elem);
//         }
//     }
//     for id in &missing {
//         elem_above_elems.remove(id);
//     }
//
//
//     // === Generate `elem_below_elems` map ===
//
//     let mut elem_below_elems : HashMap<usize,Vec<usize>> = HashMap::new();
//     for (above_id,below_ids) in &elem_above_elems {
//         for below_id in below_ids {
//             elem_below_elems.entry(*below_id).or_default().push(*above_id);
//         }
//     }
//
//
//     // === Sort indexes ===
//
//     let mut queue        = HashSet::<usize>::new();
//     let mut sorted       = vec![];
//     let mut newly_sorted = vec![];
//
//     for id in indexes {
//         if elem_above_elems.get(id).is_some() {
//             queue.insert(*id);
//         } else {
//             newly_sorted.push(*id);
//             while !newly_sorted.is_empty() {
//                 let id = newly_sorted.pop().unwrap();
//                 sorted.push(id);
//                 elem_below_elems.remove(&id).for_each(|above_ids| {
//                     for above_id in above_ids {
//                         if let Some(lst) = elem_above_elems.get_mut(&above_id) {
//                             lst.remove_item(&id);
//                             if lst.is_empty() && queue.contains(&above_id) {
//                                 queue.remove(&above_id);
//                                 newly_sorted.push(above_id);
//                             }
//                             if lst.is_empty() {
//                                 elem_above_elems.remove(&above_id);
//                             }
//                         }
//                     }
//                 })
//             }
//         }
//     }
//     sorted
// }
//
//
//
// // =============
// // === Tests ===
// // =============
//
// #[cfg(test)]
// mod sparse_tests {
//     use super::*;
//
//     #[test]
//     fn identity_with_no_rules() {
//         assert_eq!( sparse_depth_sort(&vec![]      , &default()) , Vec::<usize>::new() );
//         assert_eq!( sparse_depth_sort(&vec![1]     , &default()) , vec![1] );
//         assert_eq!( sparse_depth_sort(&vec![1,3]   , &default()) , vec![1,3] );
//         assert_eq!( sparse_depth_sort(&vec![1,2,3] , &default()) , vec![1,2,3] );
//     }
//
//     #[test]
//     fn chained_rules() {
//         let mut rules = HashMap::<usize,Vec<usize>>::new();
//         rules.insert(1,vec![2]);
//         rules.insert(2,vec![3]);
//         assert_eq!( sparse_depth_sort(&vec![]      , &rules) , Vec::<usize>::new() );
//         assert_eq!( sparse_depth_sort(&vec![1]     , &rules) , vec![1] );
//         assert_eq!( sparse_depth_sort(&vec![1,2]   , &rules) , vec![2,1] );
//         assert_eq!( sparse_depth_sort(&vec![1,2,3] , &rules) , vec![3,2,1] );
//     }
//
//     #[test]
//     fn order_preserving() {
//         let mut rules = HashMap::<usize,Vec<usize>>::new();
//         rules.insert(1,vec![2]);
//         rules.insert(2,vec![3]);
//         assert_eq!( sparse_depth_sort(&vec![10,11,12]          , &rules) , vec![10,11,12] );
//         assert_eq!( sparse_depth_sort(&vec![10,1,11,12]        , &rules) , vec![10,1,11,12] );
//         assert_eq!( sparse_depth_sort(&vec![10,1,11,2,12]      , &rules) , vec![10,11,2,1,12] );
//         assert_eq!( sparse_depth_sort(&vec![10,1,11,2,12,3,13] , &rules) , vec![10,11,12,3,2,1,13] );
//     }
// }
//
//
// #[cfg(test)]
// mod sparse_benches {
//     use super::*;
//     use test::Bencher;
//
//     /// # Results (ms)
//     ///
//     ///   iters | time(ms) |
//     ///   10^3  | 0.5      |
//     ///   10^4  | 6.3      |
//     ///   10^5  | 101.5    |
//     #[bench]
//     fn bench_ascending(b:&mut Bencher) {
//         let iters     = 1_000;
//         let ins       = (0..iters).collect_vec();
//         let out       = ins.clone();
//         let mut rules = HashMap::<usize,Vec<usize>>::new();
//         for (i,j) in out.iter().zip(out.iter().skip(1)) { rules.insert(*j,vec![*i]); }
//         b.iter(move || assert_eq!(sparse_depth_sort(&ins,&rules),out));
//     }
// }
