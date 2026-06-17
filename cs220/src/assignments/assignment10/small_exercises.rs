//! Small exercises.

use std::collections::HashSet;

use itertools::*;

use std::collections::BinaryHeap;
use std::cmp::Ordering;

/// Returns the pairs of `(i, j)` where `i < j` and `inner[i] > inner[j]` in increasing order.
///
/// For example, the inversions of `[3, 5, 1, 2, 4]` is `[(0, 2), (0, 3), (1, 2), (1, 3), (1, 4)]`
/// because as follows:
///
/// - `0 < 2`, `inner[0] = 3 > 1 = inner[2]`
/// - `0 < 3`, `inner[0] = 3 > 2 = inner[3]`
/// - `1 < 2`, `inner[1] = 5 > 1 = inner[2]`
/// - `1 < 3`, `inner[1] = 5 > 2 = inner[3]`
/// - `1 < 4`, `inner[1] = 5 > 4 = inner[4]`
///
/// Consult <https://en.wikipedia.org/wiki/Inversion_(discrete_mathematics)> for more details of inversion.
pub fn inversion<T: Ord>(inner: Vec<T>) -> Vec<(usize, usize)> {
    inner
        .iter()
        .enumerate()
        // Generate pairs for each element i with all subsequent elements j
        .flat_map(|(i, v1)| {
            inner[i + 1..]
                .iter()
                .enumerate()
                // Shift j's index to be relative to the start of the whole vector
                .map(move |(j, v2)| (i, j + i + 1, v1, v2))
        })
        // Filter only those pairs that satisfy the inversion condition
        .filter(|(_, _, v1, v2)| v1 > v2)
        // Map back to just the indices
        .map(|(i, j, _, _)| (i, j))
        .collect()
}

/// Represents a node of tree data structure.
///
/// Consult <https://en.wikipedia.org/wiki/Tree_(data_structure)> for more details on tree data structure.
#[derive(Debug)]
pub enum Node<T> {
    /// Non-leaf node
    ///
    /// It contains `(the name of node, list of child nodes)`.
    NonLeaf((T, Vec<Node<T>>)),
    /// Leaf node
    ///
    /// It contains the name of node.
    Leaf(T),
}

struct PreOrderIterator<T> {
    stack: Vec<Node<T>>
}

impl<T> Iterator for PreOrderIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let current_node = self.stack.pop()?;

        match current_node {
            Node::NonLeaf((name, childs)) => {
                for child in childs.into_iter().rev() {
                    self.stack.push(child);
                }
                Some(name)
            },
            Node::Leaf(name) => {
                Some(name)
            }
        }
    }
}

/// Traverses the tree in preorder.
///
/// The algorithm for preorder traversal is as follows:
///
/// 1. Visit the root.
/// 2. If the root is a leaf node, end the traverse.
/// 3. If the root is a non-leaf node, traverse each subtree from the child nodes.
///
/// For example, the result of preorder traversal for the following tree
///
/// ```text
///     1
///    /|\
///   2 3 4
///  /|  /|\
/// 5 6 7 8 9
/// ```
///
/// which can be represented as
///
/// ```ignore
/// Node::NonLeaf((
///     1,
///     vec![
///         Node::NonLeaf((2, vec![Node::Leaf(5), Node::Leaf(6)])),
///         Node::Leaf(3),
///         Node::NonLeaf((4, vec![Node::Leaf(7), Node::Leaf(8), Node::Leaf(9)])),
///     ]
/// ))
/// ```
///
/// is `1 -> 2 -> 5 -> 6 -> 3 -> 4 -> 7 -> 8 -> 9`.
pub fn traverse_preorder<T>(root: Node<T>) -> Vec<T> {
    // Trade-off: the original tree will be destroyed
    let it = PreOrderIterator { stack: vec![root], };
    it.collect()
}

/// File
#[derive(Debug)]
pub enum File {
    /// Directory
    ///
    /// It contains `(name of directory, list of files under the directory)`
    ///
    /// The size of a directory is the sum of the sizes of its sub-files.
    Directory(String, Vec<File>),

    /// Data
    ///
    /// It contains `(name of data, size of data)`
    Data(String, usize),
}

/// Given a file, summarize all subfiles and sizes in ascending order of size.
///
/// - Its behaviour is the same as the `du | sort -h` command on Linux.
/// - If the file size is the same, sort it by name.
/// - Assume that there are no duplicate file names.
///
/// # Example
///
/// Input:
///
/// ```txt
/// root (Directory)
/// |
/// |__a (Directory)
/// |  |__a1 (Data, size: 1)
/// |  |__a2 (Data, size: 3)
/// |
/// |__b (Directory)
/// |  |__b1 (Data, size: 3)
/// |  |__b2 (Data, size: 15)
/// |
/// |__c (Data, size: 8)
/// ```
///
/// Output: `[("a1", 1), ("a2", 3), ("b1", 3), ("a", 4), ("c", 8), ("b2", 15), ("b", 18), ("root",
/// 30)]`
pub fn du_sort(root: &File) -> Vec<(&str, usize)> {
    let mut results: Vec<(&str, usize)> = Vec::new();

    let _ = calc_file_size(root, &mut results);

    results.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(b.0)));

    results
}

fn calc_file_size<'a>(file: &'a File, results: &mut Vec<(&'a str, usize)>) -> usize {
    match file {
        File::Data(name, size) => {
            results.push((&name, *size));
            *size
        },
        File::Directory(name, childrens) => {
            let total_size = childrens.iter().map(|child| calc_file_size(child, results)).sum();
            results.push((&name, total_size));
            total_size
        }
    }
}



/// Remove all even numbers inside a vector using the given mutable reference.
/// That is, you must modify the vector using the given mutable reference instead
/// of returning a new vector.
///
/// # Example
/// ```ignore
/// let mut vec = vec![1, 2, 3, 4, 5];
/// remove_even(&mut vec);
/// assert_eq!(*vec, vec![1, 3, 5]);
/// ```
#[allow(clippy::ptr_arg)]
pub fn remove_even(inner: &mut Vec<i64>) {
    inner.retain(|&x| x % 2 == 1);
}

/// Remove all duplicate occurences of a number inside the array.
/// That is, if an integer appears more than once, remove some occurences
/// of it so that it only appears once. Note that you must modify the vector
/// using the given mutable reference instead of returning a new vector.
/// Also, note that the order does not matter.
///
/// # Example
/// ```ignore
/// let mut vec = vec![1, 2, 1, 1, 3, 7, 5, 7];
/// remove_duplicate(&mut vec);
/// assert_eq!(*vec, vec![1, 2, 3, 7, 5]);
/// ```
#[allow(clippy::ptr_arg)]
pub fn remove_duplicate(inner: &mut Vec<i64>) {
    let mut seen: HashSet<i64> = HashSet::new();
    inner.retain(|&x| seen.insert(x));
}

/// Returns the natural join of two tables using the first column as the join argument.
/// That is, for each pair of a row(`Vec<String>`) from table1 and a row(`Vec<String>`) from table2,
/// if the first element of them are equal, then add all elements of the row from table2
/// except its first element to the row from table1 and add it to the results.
/// Note that the order of results does not matter.
///
/// # Example
///
/// ```text
///        table1                     table2
/// ----------------------     ----------------------
///  20230001 |    Jack         20230001 |    CS
///  20231234 |    Mike         20230001 |    EE
///                             20231234 |    ME
///
///
///               result
/// -----------------------------------
///  20230001 |    Jack   |     CS
///  20230001 |    Jack   |     EE
///  20231234 |    Mike   |     ME
/// ```
pub fn natural_join(table1: Vec<Vec<String>>, table2: Vec<Vec<String>>) -> Vec<Vec<String>> {
    table1
        .iter()
        .flat_map(|v1| {
            table2
                .iter()
                .map(|v2| (v1[0].clone(), v2[0].clone(), v1[1].clone(), v2[1].clone()))
        })
        .filter(|(id1, id2, _, _)| id1 == id2)
        .map(|(_, id, name, dept)| Vec::from([id, name, dept]))
        .collect()
}

#[derive(Eq, PartialEq)]
struct HeapNode {
    c: u64,
    m: u64, // The critical tie-breaker tracking chronological discovery
    triple: (u64, u64, u64),
}

impl Ord for HeapNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.c.cmp(&self.c)
            .then_with(|| other.triple.0.cmp(&self.triple.0))
    }
}


impl PartialOrd for HeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// You can freely add more fields.
struct Pythagorean {
    m: u64,
    heap: BinaryHeap<HeapNode>,
}

impl Pythagorean {
    fn new() -> Self {
        Pythagorean {
            m: 2,
            heap: BinaryHeap::new(),
        }
    }
}

impl Iterator for Pythagorean {
    type Item = (u64, u64, u64);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(node) = self.heap.peek() {
                if node.c < self.m * self.m + 1 {
                    return self.heap.pop().map(|n| n.triple);
                }
            }

            let current_m = self.m;
            for n in (1..current_m).filter(|&n| (current_m - n) % 2 == 1 && gcd(current_m, n) == 1) {
                let a = current_m * current_m - n * n;
                let b = 2 * current_m * n;
                let c = current_m * current_m + n * n;
                
                self.heap.push(HeapNode {
                    c,
                    m: current_m,
                    triple: (u64::min(a, b), u64::max(a, b), c),
                });
            }

            self.m += 1;
        }
    }
}

/// Generates sequence of unique [primitive Pythagorean triples](https://en.wikipedia.org/wiki/Pythagorean_triple),
/// i.e. (a,b,c) such that a² + b² = c², a and b are coprimes, and a < b. Generate in the increasing
/// order of c.
pub fn pythagorean() -> impl Iterator<Item = (u64, u64, u64)> {
    Pythagorean::new()
}
