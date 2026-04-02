//! Collection Helper Functions

use std::collections::HashMap;
use std::hash::Hash;

/// Chunk a vector into smaller vectors of given size
///
/// # Examples
/// ```
/// use rust_core::chunk;
///
/// let items = vec![1, 2, 3, 4, 5];
/// let chunks = chunk(&items, 2);
/// assert_eq!(chunks, vec![vec![&1, &2], vec![&3, &4], vec![&5]]);
/// ```
pub fn chunk<T>(items: &[T], size: usize) -> Vec<Vec<&T>> {
    if size == 0 {
        return vec![];
    }

    items.chunks(size).map(|chunk| chunk.iter().collect()).collect()
}

/// Pluck values from a collection by key
pub fn pluck<'a, K, V>(items: &'a [HashMap<K, V>], key: &K) -> Vec<Option<&'a V>>
where
    K: Eq + Hash,
{
    items.iter().map(|item| item.get(key)).collect()
}

/// Group items by a key function
pub fn group_by<T, K, F>(items: &[T], key_fn: F) -> HashMap<K, Vec<&T>>
where
    K: Eq + Hash,
    F: Fn(&T) -> K,
{
    let mut groups: HashMap<K, Vec<&T>> = HashMap::new();

    for item in items {
        let key = key_fn(item);
        groups.entry(key).or_insert_with(Vec::new).push(item);
    }

    groups
}

/// Get first item that matches predicate
pub fn first<T, F>(items: &[T], predicate: F) -> Option<&T>
where
    F: Fn(&T) -> bool,
{
    items.iter().find(|item| predicate(item))
}

/// Get last item that matches predicate
pub fn last<T, F>(items: &[T], predicate: F) -> Option<&T>
where
    F: Fn(&T) -> bool,
{
    items.iter().rev().find(|item| predicate(item))
}

/// Check if all items match predicate
pub fn every<T, F>(items: &[T], predicate: F) -> bool
where
    F: Fn(&T) -> bool,
{
    items.iter().all(predicate)
}

/// Check if any item matches predicate
pub fn some<T, F>(items: &[T], predicate: F) -> bool
where
    F: Fn(&T) -> bool,
{
    items.iter().any(predicate)
}

/// Unique items in collection
pub fn unique<T>(items: &[T]) -> Vec<&T>
where
    T: Eq + Hash,
{
    let mut seen = HashMap::new();
    let mut result = Vec::new();

    for item in items {
        if !seen.contains_key(item) {
            seen.insert(item, ());
            result.push(item);
        }
    }

    result
}

/// Flatten nested vectors
pub fn flatten<T>(items: Vec<Vec<T>>) -> Vec<T> {
    items.into_iter().flatten().collect()
}

/// Partition items by predicate
pub fn partition<T, F>(items: Vec<T>, predicate: F) -> (Vec<T>, Vec<T>)
where
    F: Fn(&T) -> bool,
{
    items.into_iter().partition(predicate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk() {
        let items = vec![1, 2, 3, 4, 5];
        let chunks = chunk(&items, 2);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], vec![&1, &2]);
        assert_eq!(chunks[1], vec![&3, &4]);
        assert_eq!(chunks[2], vec![&5]);
    }

    #[test]
    fn test_group_by() {
        let items = vec![1, 2, 3, 4, 5, 6];
        let groups = group_by(&items, |&n| n % 2);

        assert_eq!(groups.get(&0).unwrap().len(), 3); // even numbers
        assert_eq!(groups.get(&1).unwrap().len(), 3); // odd numbers
    }

    #[test]
    fn test_first() {
        let items = vec![1, 2, 3, 4, 5];
        assert_eq!(first(&items, |&n| n > 2), Some(&3));
        assert_eq!(first(&items, |&n| n > 10), None);
    }

    #[test]
    fn test_last() {
        let items = vec![1, 2, 3, 4, 5];
        assert_eq!(last(&items, |&n| n < 4), Some(&3));
        assert_eq!(last(&items, |&n| n > 10), None);
    }

    #[test]
    fn test_every() {
        let items = vec![2, 4, 6, 8];
        assert!(every(&items, |&n| n % 2 == 0));
        assert!(!every(&items, |&n| n > 5));
    }

    #[test]
    fn test_some() {
        let items = vec![1, 3, 5, 8];
        assert!(some(&items, |&n| n % 2 == 0));
        assert!(!some(&items, |&n| n > 10));
    }

    #[test]
    fn test_unique() {
        let items = vec![1, 2, 2, 3, 3, 3, 4];
        let unique_items = unique(&items);
        assert_eq!(unique_items, vec![&1, &2, &3, &4]);
    }

    #[test]
    fn test_flatten() {
        let nested = vec![vec![1, 2], vec![3, 4], vec![5]];
        let flat = flatten(nested);
        assert_eq!(flat, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_partition() {
        let items = vec![1, 2, 3, 4, 5];
        let (even, odd) = partition(items, |&n| n % 2 == 0);
        assert_eq!(even, vec![2, 4]);
        assert_eq!(odd, vec![1, 3, 5]);
    }
}
