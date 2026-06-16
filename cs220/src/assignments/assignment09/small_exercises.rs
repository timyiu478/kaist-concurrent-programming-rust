//! Small exercises.

use std::collections::HashMap;

use itertools::Itertools;

/// Returns whether the given sequence is a fibonacci sequence starts from the given sequence's
/// first two terms.
///
/// Returns `true` if the length of sequence is less or equal than 2.
///
/// # Example
///
/// ```
/// use cs220::assignments::assignment09::small_exercises::*;
///
/// assert_eq!(is_fibonacci([1, 1, 2, 3, 5, 8, 13].into_iter()), true);
/// assert_eq!(is_fibonacci([1, 1, 2, 3, 5, 8, 14].into_iter()), false);
/// ```
pub fn is_fibonacci(inner: impl Iterator<Item = i64>) -> bool {
    let mut inner = inner;
    let mut first = match inner.next() {
        Some(val) => val,
        None => return true,
    };
    let mut second = match inner.next() {
        Some(val) => val,
        None => return true,
    };

    while let Some(item) = inner.next() {
        if let Some(expected) = first.checked_add(second) {
            if expected != item {
                return false;
            }
            (first, second) = (second, expected);
        } else {
            return false;
        }
    }

    return true
}

/// Returns the sum of `f(v)` for all element `v` the given array.
///
/// # Example
///
/// ```
/// use cs220::assignments::assignment09::small_exercises::*;
///
/// assert_eq!(sigma([1, 2].into_iter(), |x| x + 2), 7);
/// assert_eq!(sigma([1, 2].into_iter(), |x| x * 4), 12);
/// ```
pub fn sigma<T, F: Fn(T) -> i64>(inner: impl Iterator<Item = T>, f: F) -> i64 {
    let mut inner = inner;
    let mut total = 0;

    while let Some(item) = inner.next() {
        total += f(item);
    }

    total
}

/// Alternate elements from three iterators until they have run out.
///
/// You can assume that the number of elements of three iterators are same.
///
/// # Example
///
/// ```
/// use cs220::assignments::assignment09::small_exercises::*;
///
/// assert_eq!(
///     interleave3([1, 2].into_iter(), [3, 4].into_iter(), [5, 6].into_iter()),
///     vec![1, 3, 5, 2, 4, 6]
/// );
/// ```
pub fn interleave3<T>(
    list1: impl Iterator<Item = T>,
    list2: impl Iterator<Item = T>,
    list3: impl Iterator<Item = T>,
) -> Vec<T> {
    let mut list1 = list1;
    let mut list2 = list2;
    let mut list3 = list3;
    let mut v: Vec<T> = Vec::new();
    while let Some(i1) = list1.next() && let Some(i2) = list2.next() && let Some(i3) = list3.next() {
        v.push(i1);
        v.push(i2);
        v.push(i3);
    }
    v
}

/// Alternate elements from array of n iterators until they have run out.
///
/// You can assume that the number of elements of iterators are same.
///
/// # Example
///
/// ```
/// use cs220::assignments::assignment09::small_exercises::*;
///
/// assert_eq!(
///     interleave_n(&mut [[1, 2].into_iter(), [3, 4].into_iter(), [5, 6].into_iter()]),
///     vec![1, 3, 5, 2, 4, 6]
/// );
/// ```
pub fn interleave_n<T, const N: usize>(
    mut iters: [impl Iterator<Item = T>; N],
) -> impl Iterator<Item = T> {
    let mut v: Vec<T> = Vec::new();

    loop {
        for it in iters.iter_mut() {
            if let Some(item) = it.next() {
                v.push(item);
            } else {
                return v.into_iter();
            }
        }
    }
}

/// Returns mean of k smallest value's mean.
///
/// # Example
///
/// ```
/// use cs220::assignments::assignment09::small_exercises::*;
///
/// assert_eq!(
///     k_smallest_mean(vec![1, 3, 2].into_iter(), 2),
///     ((1 + 2) as f64 / 2.0)
/// );
/// assert_eq!(
///     k_smallest_mean(vec![7, 5, 3, 6].into_iter(), 3),
///     ((3 + 5 + 6) as f64 / 3.0)
/// );
/// ```
pub fn k_smallest_mean(inner: impl Iterator<Item = i64>, k: usize) -> f64 {
    let mut v: Vec<i64> = inner.collect();

    v.sort();

    let actual_k = std::cmp::min(k, v.len());

    if actual_k == 0 {
        return 0.0;
    }

    let slice = &v[..actual_k];

    slice.iter().map(|&x| x as f64).sum::<f64>() / actual_k as f64
}

/// Returns mean for each class.
///
/// # Example
///
/// ```
/// use cs220::assignments::assignment09::small_exercises::*;
///
/// assert_eq!(
///     calculate_mean(
///         [
///             ("CS100".to_string(), 60),
///             ("CS200".to_string(), 60),
///             ("CS200".to_string(), 80),
///             ("CS300".to_string(), 100),
///         ]
///         .into_iter()
///     ),
///     [
///         ("CS100".to_string(), 60.0),
///         ("CS200".to_string(), 70.0),
///         ("CS300".to_string(), 100.0)
///     ]
///     .into_iter()
///     .collect()
/// );
/// ```
pub fn calculate_mean(inner: impl Iterator<Item = (String, i64)>) -> HashMap<String, f64> {
    let mut inner = inner;
    let mut hm: HashMap<String, (i64, i64)> = HashMap::new();

    while let Some(item) = inner.next() {
        let (score, count) = hm.get(&item.0).copied().unwrap_or((0, 0));
        let _ = hm.insert(item.0, (score + item.1, count + 1));
    }

    let mean: HashMap<String, f64> = hm
        .into_iter()
        .map(|(key, value)| (key, value.0 as f64 / value.1 as f64))
        .collect();

    mean
}

/// Among the cartesian product of input vectors, return the number of sets whose sum equals `n`.
///
/// # Example
///
/// The cartesian product of [1, 2, 3] and [2, 3] are:
///     [1, 2], [1, 3], [2, 2], [2, 3], [3, 2], [3, 3].
///
/// Among these sets, the number of sets whose sum is 4 is 2, which is [1, 3] and [2, 2].
///
/// ```
/// use cs220::assignments::assignment09::small_exercises::*;
///
/// assert_eq!(sum_is_n(vec![vec![1, 2, 3], vec![2, 3]], 3), 1);
/// assert_eq!(sum_is_n(vec![vec![1, 2, 3], vec![2, 3]], 4), 2);
/// assert_eq!(sum_is_n(vec![vec![1, 2, 3], vec![2, 3]], 5), 2);
/// assert_eq!(sum_is_n(vec![vec![1, 2, 3], vec![2, 3]], 6), 1);
/// assert_eq!(sum_is_n(vec![vec![1, 2, 3], vec![2, 3]], 2), 0);
/// ```
pub fn sum_is_n(inner: Vec<Vec<i64>>, n: i64) -> usize {
    // Handle the edge case where the input list of vectors is completely empty
    if inner.is_empty() {
        return if n == 0 { 1 } else { 0 };
    }

    // 1. Seed our tracking map with a single sum of 0, which has occurred 1 time.
    let mut current_sums = HashMap::new();
    let _ = current_sums.insert(0, 1);

    // 2. Process each vector one by one, rolling the new combinations forward
    for vec in inner {
        let mut next_sums = HashMap::new();

        for (&running_sum, &count) in &current_sums {
            for &num in &vec {
                let new_sum = running_sum + num;
                
                // Add the combinations up into our new temporary map
                *next_sums.entry(new_sum).or_insert(0) += count;
            }
        }

        // Overwrite our main tracker with the newly generated layer of sums
        current_sums = next_sums;
    }

    // 3. Return the exact count for the target sum `n`
    current_sums.get(&n).copied().unwrap_or(0)
}

/// Returns a new vector that contains the item that appears `n` times in the input vector in
/// increasing order.
///
/// # Example
///
/// ```
/// use cs220::assignments::assignment09::small_exercises::*;
///
/// assert_eq!(find_count_n(vec![1, 2], 1), vec![1, 2]);
/// assert_eq!(find_count_n(vec![1, 3, 3], 1), vec![1]);
/// assert_eq!(find_count_n(vec![1, 3, 3], 2), vec![3]);
/// assert_eq!(find_count_n(vec![1, 2, 3, 4, 4], 1), vec![1, 2, 3]);
/// ```
pub fn find_count_n(inner: Vec<usize>, n: usize) -> Vec<usize> {
    let mut count_n: HashMap<usize, usize> = HashMap::new();

    // Count occurrences of each number
    inner.iter().for_each(|&x| *count_n.entry(x).or_insert(0) += 1);

    // Filter by count, extract the key, and collect into a vector
    let mut result: Vec<usize> = count_n
        .into_iter()
        .filter(|&(_, value)| value == n)
        .map(|(key, _)| key)
        .collect();

    // Sort the vector to ensure increasing order
    result.sort_unstable();

    result
}

/// Return the position of the median element in the vector.
///
/// For a data set `x` of `n` elements, the median can be defined as follows:
///
/// - If `n` is odd, the median is `(n+1)/2`-th smallest element of `x`.
/// - If `n` is even, the median is `(n/2)+1`-th smallest element of `x`.
///
/// Please following these rules:
///
/// - If the list is empty, returns `None`.
/// - If several elements are equally median, the position of the first of them is returned.
///
/// # Example
///
/// ```
/// use cs220::assignments::assignment09::small_exercises::*;
///
/// assert_eq!(position_median(vec![1, 3, 3, 6, 7, 8, 9]), Some(3));
/// assert_eq!(position_median(vec![1, 3, 3, 3]), Some(1));
/// ```
pub fn position_median<T: Ord>(inner: Vec<T>) -> Option<usize> {
    let len = inner.len(); // 1. Save the length before consuming `inner`

    if len == 0 {
        return None;
    }

    // 2. Consume `inner` into an enumerated vector
    let mut v: Vec<(usize, T)> = inner.into_iter().enumerate().collect();

    // 3. Sort stably by the value (the second element of the tuple)
    v.sort_by(|a, b| a.1.cmp(&b.1));

    // 4. Calculate the correct 0-based target index
    let target_idx = if len % 2 == 0 {
        ((len / 2) + 1) - 1 // (n/2)+1-th element maps to index (len/2)
    } else {
        ((len + 1) / 2) - 1 // (n+1)/2-th element maps to index (len-1)/2
    };

    // 5. Get the actual median value
    let median_value = &v[target_idx].1;

    // 6. Scan the sorted array (or look back) to find the original index 
    // of the FIRST occurrence of this value.
    v.iter()
     .filter(|(_, val)| val == median_value)
     .map(|(orig_idx, _)| *orig_idx)
     .min() // Gets the smallest original index among duplicates
}

/// Returns the sum of all elements in a two-dimensional array.
///
/// # Example
/// ```
/// use cs220::assignments::assignment09::small_exercises::*;
///
/// assert_eq!(
///     two_dimensional_sum([[1, 2, 3].into_iter(), [4, 5, 6].into_iter()].into_iter()),
///     21
/// );
/// ```
pub fn two_dimensional_sum(inner: impl Iterator<Item = impl Iterator<Item = i64>>) -> i64 {
    inner.map(|it| it.sum::<i64>()).sum()
}

/// Returns whether the given string is palindrome or not.
///
/// A palindrome is a word, number, phrase, or other sequence of characters which reads the same
/// backward as forward.
///
/// We consider the empty string as a palindrome.
///
/// Consult <https://en.wikipedia.org/wiki/Palindrome>.
pub fn is_palindrome(s: String) -> bool {
    s.chars().eq(s.chars().rev())
}
