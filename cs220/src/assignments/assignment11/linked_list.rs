//! Singly linked list.
//!
//! Consult <https://doc.rust-lang.org/book/ch15-01-box.html>.

use std::fmt::Debug;

/// Node of the list.
#[derive(Debug)]
pub struct Node<T: Debug> {
    /// Value of current node.
    pub value: T,

    /// Pointer to the next node. If it is `None`, there is no next node.
    pub next: Option<Box<Node<T>>>,
}

impl<T: Debug> Node<T> {
    /// Creates a new node.
    pub fn new(value: T) -> Self {
        Self { value, next: None }
    }
}

/// A singly-linked list.
#[derive(Debug)]
pub struct SinglyLinkedList<T: Debug> {
    /// Head node of the list. If it is `None`, the list is empty.
    head: Option<Node<T>>,
}

impl<T: Debug> Default for SinglyLinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Debug> SinglyLinkedList<T> {
    /// Creates a new list.
    pub fn new() -> Self {
        Self { head: None }
    }

    /// Adds the given node to the front of the list.
    pub fn push_front(&mut self, value: T) {
        let mut node = Node::new(value);
        // .take(): takes the value out of the option, leaving a None in its place.
        // it is needed to ensure the old_head has single owner at all time
        if let Some(old_head) = self.head.take() {
            node.next = Some(Box::new(old_head));
        }
        self.head = Some(node);
    }

    /// Adds the given node to the back of the list.
    pub fn push_back(&mut self, value: T) {
        let new_node = Node::new(value);

        if self.head.is_none() {
            self.head = Some(new_node);
        } else {
            // We get a mutable reference to the internal Node inside self.head.
            let mut current = self.head.as_mut().unwrap();

            // Traverse using Box's deref coercion
            while let Some(ref mut next_box) = current.next {
                // next_box is &mut Box<Node<T>>, which automatically 
                // coerces down into a standard &mut Node<T>
                current = next_box; 
            }

            current.next = Some(Box::new(new_node));
        }
    }

    /// Removes and returns the node at the front of the list.
    pub fn pop_front(&mut self) -> Option<T> {
        let old_head = self.head.take()?;
        // self.head is None now

        if let Some(next_node) = old_head.next {
            self.head = Some(*next_node);
        }

        Some(old_head.value)
    }

    /// Removes and returns the node at the back of the list.
    pub fn pop_back(&mut self) -> Option<T> {
        // 1. If the list is empty, return None immediately
        if self.head.is_none() {
            return None;
        }

        // 2. If the list has exactly 1 element, pop the head
        if self.head.as_ref().map_or(false, |node| node.next.is_none()) {
            return self.head.take().map(|node| node.value);
        }

        // 3. Safe look-ahead loop for 2+ elements
        let mut current = self.head.as_mut().unwrap();
        while current.next.as_ref().and_then(|n| n.next.as_ref()).is_some() {
            current = current.next.as_mut().unwrap();
        }

        // 4. Pluck the last node out
        current.next.take().map(|boxed_node| boxed_node.value)
    }

    /// Create a new list from the given vector `vec`.
    pub fn from_vec(vec: Vec<T>) -> Self {
        let mut ll = SinglyLinkedList::new();

        for item in vec.into_iter().rev() {
            ll.push_front(item);
        }

        ll
    }

    /// Convert the current list into a vector.
    pub fn into_vec(self) -> Vec<T> {
        let mut v = Vec::new();

        let mut current = self.head;

        while let Some(node) = current {
            v.push(node.value);
            // unbox
            current = node.next.map(|b| *b);
        }

        v
    }

    /// Return the length (i.e., number of nodes) of the list.
    pub fn length(&self) -> usize {
        let mut length = 0;

        let mut current = self.head.as_ref();

        while let Some(node) = current {
            length += 1;
        
            // .as_deref() converts &Option<Box<Node<T>>> into Option<&Node<T>>
            current = node.next.as_deref();
        }

        length
    }

    /// Apply function `f` on every element of the list.
    ///
    /// # Examples
    ///
    /// `self`: `[1, 2]`, `f`: `|x| x + 1` ==> `[2, 3]`
    pub fn map<F: Fn(T) -> T>(self, f: F) -> Self {
        let flat_vector = self.into_vec();

        let mapped_vector: Vec<T> = flat_vector.into_iter().map(f).collect();

        Self::from_vec(mapped_vector)
    }

    /// Apply given function `f` for each adjacent pair of elements in the list.
    /// If `self.length() < 2`, do nothing.
    ///
    /// # Examples
    ///
    /// `self`: `[1, 2, 3, 4]`, `f`: `|x, y| x + y`
    /// // each adjacent pair of elements: `(1, 2)`, `(2, 3)`, `(3, 4)`
    /// // apply `f` to each pair: `f(1, 2) == 3`, `f(2, 3) == 5`, `f(3, 4) == 7`
    /// ==> `[3, 5, 7]`
    pub fn pair_map<F: Fn(T, T) -> T>(self, f: F) -> Self
    where
        T: Clone,
    {
        let vec = self.into_vec();
        let it1 = vec.clone().into_iter();
        let mut it2 = vec.into_iter();

        drop(it2.next()); // skip the 1st element

        let mapped_vector: Vec<T> = it1
            .zip(it2)
            .map(|(x, y)| f(x,y))
            .collect();

        Self::from_vec(mapped_vector)
    }
}

// A list of lists.
impl<T: Debug> SinglyLinkedList<SinglyLinkedList<T>> {
    /// Flatten the list of lists into a single list.
    ///
    /// # Examples
    /// `self`: `[[1, 2, 3], [4, 5, 6], [7, 8]]`
    /// ==> `[1, 2, 3, 4, 5, 6, 7, 8]`
    pub fn flatten(self) -> SinglyLinkedList<T> {
        let list_of_list_vec = self.into_vec();

        let single_linked_list: Vec<T> = list_of_list_vec
            .into_iter()
            .flat_map(|ll| ll.into_vec().into_iter())
            .collect();

        SinglyLinkedList::from_vec(single_linked_list)
    }
}
