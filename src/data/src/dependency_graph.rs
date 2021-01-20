//! A dependency graph implementation optimized for sorting depth-indexed components.

use crate::prelude::*;

use std::collections::BTreeSet;



// ============
// === Node ===
// ============

/// A dependency graph node. Registers all incoming and outgoing edges.
#[derive(Clone,Debug)]
#[derive(Derivative)]
#[derivative(Default(bound=""))]
#[allow(missing_docs)]
pub struct Node<T> {
    pub ins : Vec<T>,
    pub out : Vec<T>,
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
#[derive(Clone)]
#[derive(Derivative)]
#[derivative(Default(bound="T:Eq+Hash+Ord"))]
#[derivative(Debug(bound="T:Debug+Eq+Hash"))]
pub struct DependencyGraph<T> {
    nodes : BTreeMap<T,Node<T>>
}

impl<T:Clone+Eq+Hash+Ord> DependencyGraph<T> {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }

    /// Insert a new dependency to the graph.
    pub fn insert_dependency(&mut self, first:T, second:T) {
        let first_key  = first.clone();
        let second_key = second.clone();
        self.nodes.entry(first_key).or_default().out.push(second);
        self.nodes.entry(second_key).or_default().ins.push(first);
    }

    /// Removes all dependencies from nodes whose indexes do not belong to the provided slice.
    pub fn keep_only(&mut self, keys:&[T]) {
        self.unchecked_keep_only(&keys.iter().cloned().sorted().rev().collect_vec())
    }

    /// Removes all dependencies from nodes whose indexes do not belong to the provided slice.
    pub fn kept_only(mut self, keys:&[T]) -> Self {
        self.keep_only(keys);
        self
    }

    /// Just like [`keep_only`], but the provided slice must be sorted in reversed order.
    pub fn unchecked_keep_only(&mut self, rev_sorted_keys:&[T]) {
        let mut keep         = rev_sorted_keys.to_vec();
        let mut next_to_keep = keep.pop();
        let     keys         = self.nodes.keys().cloned().collect_vec();
        let mut keys_iter    = keys.iter();
        let mut opt_key      = keys_iter.next();
        loop {
            match opt_key {
                None => break,
                Some(key) => {
                    match next_to_keep.as_ref().map(|t| t.cmp(&key)) {
                        Some(std::cmp::Ordering::Less) => {
                            next_to_keep = keep.pop()
                        },
                        Some(std::cmp::Ordering::Equal) => {
                            next_to_keep = keep.pop();
                            opt_key      = keys_iter.next();
                        }
                        _ => {
                            if let Some(node) = self.nodes.get_mut(&key) {
                                let node = mem::take(node);
                                for key2 in node.ins {
                                    self.nodes.get_mut(&key2).for_each(|t| t.out.remove_item(&key))
                                }
                                for key2 in node.out {
                                    self.nodes.get_mut(&key2).for_each(|t| t.ins.remove_item(&key))
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Just like [`kept_only`], but the provided slice must be sorted in reversed order.
    pub fn unchecked_kept_only(mut self, rev_sorted_keys:&[T]) -> Self {
        self.unchecked_keep_only(rev_sorted_keys);
        self
    }

    /// Sorts the provided indexes in topological order based on the rules recorded in the graph.
    /// In case the graph is not a DAG, it will still be sorted by breaking cycles on elements with
    /// the smallest index.
    pub fn topo_sort(&self, keys:&[T]) -> Vec<T> {
        self.unchecked_topo_sort(keys.iter().cloned().sorted().rev().collect_vec())
    }

    /// Just like [`topo_sort`], but the provided slice must be sorted in reversed order.
    pub fn unchecked_topo_sort(&self, rev_sorted_keys:Vec<T>) -> Vec<T> {
        let mut sorted      = Vec::<T>::new();
        let mut orphans     = BTreeSet::<T>::new();
        let mut non_orphans = BTreeSet::<T>::new();
        let this            = self.clone().unchecked_kept_only(&rev_sorted_keys);
        sorted.reserve_exact(rev_sorted_keys.len());

        let mut nodes = this.nodes;
        for key in rev_sorted_keys.into_iter() {
            let ins_empty = nodes.get(&key).map(|t|t.ins.is_empty()) != Some(false);
            if ins_empty { orphans.insert(key); }
            else         { non_orphans.insert(key); }
        }

        loop {
            match orphans.iter().next().cloned() {
                None => {
                    match non_orphans.iter().next().cloned() {
                        None => break,
                        Some(ix) => {
                            // NON DAG
                            non_orphans.remove(&ix);
                            orphans.insert(ix);
                        }
                    }
                },
                Some(ix) => {
                    sorted.push(ix.clone());
                    orphans.remove(&ix);
                    if let Some(node) = nodes.get_mut(&ix) {
                        for ix2 in mem::take(&mut node.out) {
                            if let Some(node2) = nodes.get_mut(&ix2) {
                                let ins = &mut node2.ins;
                                ins.remove_item(&ix);
                                if ins.is_empty() && non_orphans.remove(&ix2) {
                                    orphans.insert(ix2);
                                }
                            }
                        }
                    }
                }
            }
        }
        sorted
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
pub fn assert_valid_sort(graph:&DependencyGraph<usize>, sorted:&[usize]) {
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
    ///   10^3  | 0.47     |
    ///   10^4  | 5.2      |
    ///   10^5  | 74.2     |
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
    ///   10^3  | 0.5      |
    ///   10^4  | 6.2      |
    ///   10^5  | 86.8     |
    #[bench]
    fn bench_descending(b:&mut Bencher) {
        let iters     = 1_000;
        let out       = (0..iters).rev().collect_vec();
        let mut graph = DependencyGraph::new();
        for (i,j) in out.iter().zip(out.iter().skip(1)) { graph.insert_dependency(*i,*j); }
        b.iter(move || assert_eq!(graph.topo_sort(&out),out));
    }
}
