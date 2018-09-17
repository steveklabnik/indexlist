//! A doubly linked list, backed by a vector.
//!
//! This crate provides a struct, `IndexList<T>`, which is a doubly-linked
//! list. However, unlike a traditional linked list, which heap allocates
//! each of its nodes individually, all nodes are stored in a vector. Rather
//! than provide pointers to nodes, an `Index` struct can be used to access
//! a particular element in the middle of the list.
//!
//! # Safety
//!
//! This crate uses `#![deny(unsafe_code)]` to ensure everything is implemented
//! in 100% Safe Rust.
//!
//! # Generational indexes
//!
//! `Index` uses a generations scheme, so that if you hold an `Index` to a node,
//! and it's removed, and a new node is allocated in its place, you do not access
//! the new node.
//!
//! # Performance
//!
//! In general, performance is quite good. Benchmarks against the standard library's
//! `LinkedList<T>` are provided. But some other details:
//!
//! * The list keeps track of its head and tail for efficient insertion.
//! * The underlying vector only grows, never shrinks. When a node is removed, its
//!   entry is marked as free for future insertions.
//! * Free entries are themselves kept as a singly-linked list, meaning that they
//!   can be re-used efficiently.
//!
//! # Missing features
//!
//! Right now, I've only implemented a minimal number of features; there's `iter`
//! but no `into_iter` and `iter_mut`. This is on the to-do list. PRs welcome!
//!
//! # Examples
//!
//! Creating a list, appending nodes, and printing them out:
//!
//! ```
//! extern crate indexlist;
//!
//! use indexlist::IndexList;
//!
//! let mut list = IndexList::new();
//!
//! list.push_back(5);
//! list.push_back(10);
//! list.push_back(15);
//!
//! // This prints 5, 10, and then 15, each on its own line
//! for element in list.iter() {
//!     println!("{}", element);
//! }
//! ```
//!
//! Removing an item from the list:
//!
//! ```
//! extern crate indexlist;
//!
//! use indexlist::IndexList;
//!
//! let mut list = IndexList::new();
//!
//! let five = list.push_back(5);
//! list.push_back(10);
//!
//! list.remove(five);
//!
//! // 5 is no longer in the list
//! assert!(list.get(five).is_none());
//! ```
//!
//! Generational indexes:
//!
//! ```
//! extern crate indexlist;
//!
//! use indexlist::IndexList;
//!
//! let mut list = IndexList::new();
//!
//! let five = list.push_back(5);
//! list.push_back(10);
//!
//! list.remove(five);
//!
//! // since we have a free spot, this will go where 5 was
//! list.push_back(15);
//!
//! // our index is out of date, and so will not return 15 here
//! assert!(list.get(five).is_none());
//! ```

#![deny(unsafe_code)]

/// A doubly linked list, backed by a vector.
///
/// See the crate documentation for more.
#[derive(Debug, PartialEq)]
pub struct IndexList<T> {
    contents: Vec<Entry<T>>,
    generation: usize,
    next_free: Option<usize>,
    head: Option<usize>,
    tail: Option<usize>,
}

#[derive(Debug, PartialEq)]
enum Entry<T> {
    Free { next_free: Option<usize> },
    Occupied(OccupiedEntry<T>),
}

#[derive(Debug, PartialEq)]
struct OccupiedEntry<T> {
    item: T,
    generation: usize,
    next: Option<usize>,
    prev: Option<usize>,
}

/// A reference to an element in the list.
///
/// If you have an `Index`, you can get or remove the item at that position in
/// the list.
///
/// # Generational indexes
///
/// `Index` employs a "generational index" scheme. A "generation" is a counter,
/// saved by the `IndexList<T>`. This counter increases whenever an item is
/// removed from the list. Each item in the list keeps track of the generation
/// it was inserted in.
///
/// An Index also keeps track of a generation. When you attempt to manipulate an
/// item in the list via an `Index`, the generations are compared. If the
/// `Index`'s generation is older than the item at that position, it is stale,
/// and so that item will not be returned or removed.
///
/// This scheme lets us re-use removed slots in the list, while ensuring that
/// you won't see bad data.
///
/// # Examples
///
/// You can get an `Index` by inserting something into the list:
///
/// ```
/// extern crate indexlist;
///
/// use indexlist::IndexList;
///
/// let mut list = IndexList::new();
///
/// // this is an Index
/// let index = list.push_back(5);
/// ```
///
/// You can also get one with `index_of`:
///
/// ```
/// extern crate indexlist;
///
/// use indexlist::IndexList;
///
/// let mut list = IndexList::new();
///
/// let five = list.push_back(5);
///
/// let index = list.index_of(&5);
///
/// assert_eq!(Some(five), index);
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Index {
    index: usize,
    generation: usize,
}

impl<T> IndexList<T>
where
    T: PartialEq,
    T: std::fmt::Debug,
{
    /// Creates a new `IndexList<T>`.
    ///
    /// # Examples
    ///
    /// Making a new list:
    ///
    /// ```
    /// extern crate indexlist;
    ///
    /// use indexlist::IndexList;
    ///
    /// let list: IndexList<i32> = IndexList::new();
    /// ```
    pub fn new() -> IndexList<T> {
        IndexList {
            contents: Vec::new(),
            generation: 0,
            next_free: None,
            head: None,
            tail: None,
        }
    }

    /// Creates a new `IndexList<T>` with a given capacity.
    ///
    /// If you know roughly how many elements will be stored in the list,
    /// creating one with that capacity can reduce allocations, increasing
    /// performance.
    ///
    /// # Examples
    ///
    /// Making a new list:
    ///
    /// ```
    /// extern crate indexlist;
    ///
    /// use indexlist::IndexList;
    ///
    /// let list: IndexList<i32> = IndexList::with_capacity(100);
    /// ```
    pub fn with_capacity(size: usize) -> IndexList<T> {
        IndexList {
            contents: Vec::with_capacity(size),
            generation: 0,
            next_free: None,
            head: None,
            tail: None,
        }
    }

    /// Returns a reference to the first item in the list.
    ///
    /// Will return `None` if the list is empty.
    ///
    /// # Examples
    ///
    /// The first item is often the first one that's pushed on:
    ///
    /// ```
    /// extern crate indexlist;
    ///
    /// use indexlist::IndexList;
    ///
    /// let mut list = IndexList::new();
    ///
    /// list.push_back(5);
    ///
    /// assert_eq!(list.head(), Some(&5));
    /// ```
    ///
    /// But of course, not always!
    ///
    /// ```
    /// extern crate indexlist;
    ///
    /// use indexlist::IndexList;
    ///
    /// let mut list = IndexList::new();
    ///
    /// list.push_back(5);
    ///
    /// // this will append to the front, so it's now the head
    /// list.push_front(10);
    ///
    /// assert_eq!(list.head(), Some(&10));
    /// ```
    pub fn head(&self) -> Option<&T> {
        let index = self.head?;

        self.contents.get(index).and_then(|e| match e {
            Entry::Free { .. } => None,
            Entry::Occupied(e) => Some(&e.item),
        })
    }

    /// Adds this item to the tail of the list.
    ///
    /// # Examples
    ///
    /// Pushing several numbers into a list:
    ///
    /// ```
    /// extern crate indexlist;
    ///
    /// use indexlist::IndexList;
    ///
    /// let mut list = IndexList::new();
    ///
    /// list.push_back(5);
    /// list.push_back(10);
    /// list.push_back(15);
    ///
    /// // This prints 5, 10, and then 15, each on its own line
    /// for element in list.iter() {
    ///     println!("{}", element);
    /// }
    /// ```
    pub fn push_back(&mut self, item: T) -> Index {
        // first, we need to find a suitable place to insert

        // is the head of the list empty? If so, that's easy.
        if self.head.is_none() {
            let generation = self.generation;

            let index = if let Some(index) = self.next_free {
                match self.contents[index] {
                    Entry::Occupied { .. } => panic!("Corrupted list"),
                    Entry::Free { next_free } => self.next_free = next_free,
                }

                self.contents[index] = Entry::Occupied(OccupiedEntry {
                    item,
                    generation,
                    next: None,
                    prev: None,
                });

                index
            } else {
                let index = self.contents.len();

                self.contents.push(Entry::Occupied(OccupiedEntry {
                    item,
                    generation,
                    next: None,
                    prev: None,
                }));

                index
            };

            self.tail = Some(index);
            self.head = Some(index);

            return Index { index, generation };
        }

        // if it isn't empty, then we need to check the free list and put our
        // new item in the proper place

        // we have a tail, so we can unwrap; we need this for appending
        let tail_index = self.tail.unwrap();

        let position = if let Some(position) = self.next_free {
            // update next_free
            match self.contents[position] {
                Entry::Occupied { .. } => panic!("Corrupted list"),
                Entry::Free { next_free } => self.next_free = next_free,
            }

            self.contents[position] = Entry::Occupied(OccupiedEntry {
                item,
                generation: self.generation,
                next: None,
                prev: Some(tail_index),
            });

            position
        } else {
            // we don't have any, so append to the end of the list
            let position = self.contents.len();

            self.contents.push(Entry::Occupied(OccupiedEntry {
                item,
                generation: self.generation,
                next: None,
                prev: Some(tail_index),
            }));

            position
        };

        // and then fix up the tail to refer to it
        let new_index = Index {
            index: position,
            generation: self.generation,
        };

        // we found this index before so we know it exists
        match &mut self.contents[tail_index] {
            Entry::Free { .. } => panic!("Corrupted list"),
            Entry::Occupied(e) => e.next = Some(new_index.index),
        }

        // update our tail to properly point at the newly inserted element
        self.tail = Some(position);

        // and finally, return the index associated with our new tail
        new_index
    }

    /// Adds this item to the head of the list.
    ///
    /// # Examples
    ///
    /// Pushing several numbers into a list:
    ///
    /// ```
    /// extern crate indexlist;
    ///
    /// use indexlist::IndexList;
    ///
    /// let mut list = IndexList::new();
    ///
    /// list.push_front(5);
    /// list.push_front(10);
    /// list.push_front(15);
    ///
    /// // This prints 15, 10, and then 5, each on its own line
    /// for element in list.iter() {
    ///     println!("{}", element);
    /// }
    /// ```
    pub fn push_front(&mut self, item: T) -> Index {
        // first, we need to find a suitable place to insert

        // is the head of the list empty? If so, that's easy.
        if self.head.is_none() {
            self.contents.push(Entry::Occupied(OccupiedEntry {
                item,
                generation: self.generation,
                next: None,
                prev: None,
            }));

            self.tail = Some(0);
            self.head = Some(0);

            return Index {
                index: 0,
                generation: self.generation,
            };
        }

        // if it isn't empty, then we need to check the free list and put our
        // new item in the proper place

        // we have a head, so we can unwrap; we need this for appending
        let head_index = self.head.unwrap();

        let position = if let Some(position) = self.next_free {
            self.contents[position] = Entry::Occupied(OccupiedEntry {
                item,
                generation: self.generation,
                next: Some(head_index),
                prev: None,
            });

            position
        } else {
            // we don't have any, so append to the end of the list
            let position = self.contents.len();

            self.contents.push(Entry::Occupied(OccupiedEntry {
                item,
                generation: self.generation,
                next: Some(head_index),
                prev: None,
            }));

            position
        };

        let new_index = Index {
            index: position,
            generation: self.generation,
        };

        // and then fix up the head to refer to it

        // we found this index before so we know it exists
        match &mut self.contents[head_index] {
            Entry::Free { .. } => panic!("Corrupted list"),
            Entry::Occupied(e) => e.prev = Some(new_index.index),
        }

        // update our head to properly point at the newly inserted element
        self.head = Some(position);

        // and finally, return the index associated with our new tail
        new_index
    }

    /// Does this list contain this element?
    ///
    /// Returns true if it does, and false if it does not.
    ///
    /// # Examples
    ///
    /// Checking both possibilities:
    ///
    /// ```
    /// extern crate indexlist;
    ///
    /// use indexlist::IndexList;
    ///
    /// let mut list = IndexList::new();
    ///
    /// list.push_back(5);
    ///
    /// // our list does contain five
    /// assert!(list.contains(&5));
    ///
    /// // our list does not contain ten
    /// assert!(!list.contains(&10));
    /// ```
    pub fn contains(&self, value: &T) -> bool {
        self.iter().any(|e| e == value)
    }

    /// Returns the item at this index if it exists.
    ///
    /// If there's an item at this index, then this will return a reference to
    /// it. If not, returns `None`.
    ///
    /// Indexes are generational, and so this method will use the generation to
    /// determine if this element exists. For more, see [`Index`'s documentation].
    ///
    /// [`Index`'s documentation]: struct.Index.html
    ///
    /// # Examples
    ///
    /// Getting an element at an index:
    ///
    /// ```
    /// extern crate indexlist;
    ///
    /// use indexlist::IndexList;
    ///
    /// let mut list = IndexList::new();
    ///
    /// let five = list.push_back(5);
    ///
    /// assert_eq!(list.get(five), Some(&5));
    /// ```
    ///
    /// An element that doesn't exist returns `None`:
    ///
    /// ```
    /// extern crate indexlist;
    ///
    /// use indexlist::IndexList;
    ///
    /// let mut list = IndexList::new();
    ///
    /// let five = list.push_back(5);
    ///
    /// list.remove(five);
    ///
    /// assert!(list.get(five).is_none());
    /// ```
    ///
    /// Generational indexes ensure that we don't access incorrect items:
    ///
    /// ```
    /// extern crate indexlist;
    ///
    /// use indexlist::IndexList;
    ///
    /// let mut list = IndexList::new();
    ///
    /// let five = list.push_back(5);
    /// list.push_back(10);
    ///
    /// list.remove(five);
    ///
    /// // since we have a free spot, this will go where 5 was
    /// list.push_back(15);
    ///
    /// // our index is out of date, and so will not return 15 here
    /// assert!(list.get(five).is_none());
    /// ```
    pub fn get(&self, index: Index) -> Option<&T> {
        match self.contents.get(index.index)? {
            Entry::Occupied(e) if e.generation == index.generation => Some(&e.item),
            _ => None,
        }
    }

    /// Removes the item at this index, and returns the removed item.
    ///
    /// If there's an item at this index, then this will remove it from the list
    /// and return it. If there isn't, it will return `None`.
    ///
    /// Indexes are generational, and so this method will use the generation to
    /// determine if this element exists. For more, see [`Index`'s documentation].
    ///
    /// [`Index`'s documentation]: struct.Index.html
    ///
    /// # Examples
    ///
    /// Removing an element from an index:
    ///
    /// ```
    /// extern crate indexlist;
    ///
    /// use indexlist::IndexList;
    ///
    /// let mut list = IndexList::new();
    ///
    /// let five = list.push_back(5);
    ///
    /// let five = list.remove(five);
    /// assert_eq!(five, Some(5));
    ///
    /// assert!(!list.contains(&5));
    /// ```
    ///
    /// An element that doesn't exist returns `None`:
    ///
    /// ```
    /// extern crate indexlist;
    ///
    /// use indexlist::IndexList;
    ///
    /// let mut list = IndexList::new();
    ///
    /// let five = list.push_back(5);
    ///
    /// list.remove(five);
    ///
    /// assert!(list.remove(five).is_none());
    /// ```
    ///
    /// Generational indexes ensure that we don't access incorrect items:
    ///
    /// ```
    /// extern crate indexlist;
    ///
    /// use indexlist::IndexList;
    ///
    /// let mut list = IndexList::new();
    ///
    /// let five = list.push_back(5);
    /// list.push_back(10);
    ///
    /// list.remove(five);
    ///
    /// // since we have a free spot, this will go where 5 was
    /// list.push_back(15);
    ///
    /// // our index is out of date, and so will not return 15 here
    /// assert!(list.remove(five).is_none());
    /// ```
    pub fn remove(&mut self, index: Index) -> Option<T> {
        // if we have no head or tail, then we have an emtpy list, so return
        let head_index = self.head?;
        let tail_index = self.tail?;

        // we want to do just get, but then we run into borrowing issues.
        //
        // we could implement Entry, but... ugh. So let's fetch just the indexes for now.
        let (prev_index, index, next_index) = match self.contents.get(index.index)? {
            Entry::Free { .. } => return None,
            Entry::Occupied(e) => {
                // are we of the right generation?
                if index.generation != e.generation {
                    return None;
                }

                (e.prev, index.index, e.next)
            }
        };

        let removed = std::mem::replace(
            // we know this index is valid thanks to the get on line 224
            &mut self.contents[index],
            Entry::Free {
                next_free: self.next_free,
            },
        );

        // update our free list to point to this new space
        self.next_free = Some(index);

        // when we remove a node, we need to increase the generation to invalidate
        // older indexes that may be refering to this spot
        self.generation += 1;

        // now we need to fix up any next or previous nodes. we have four cases:
        //
        // * index is at the head and tail (only item in the list)
        // * index is at the head
        // * index is at the tail
        // * index is in the middle

        // index is at the head and tail (only item in the list)
        if (index == head_index) && (index == tail_index) {
            self.head = None;
            self.tail = None;

        // index is at the head
        } else if index == head_index {
            let next = match &mut self.contents[next_index.unwrap()] {
                Entry::Free { .. } => panic!("Corrupted list"),
                Entry::Occupied(e) => e,
            };

            next.prev = None;
            self.head = next_index;

        // index is at the tail
        } else if index == tail_index {
            let prev = match &mut self.contents[prev_index.unwrap()] {
                Entry::Free { .. } => panic!("Corrupted list"),
                Entry::Occupied(e) => e,
            };

            prev.next = None;
            self.tail = prev_index;

        // index is in the middle
        } else if index != head_index && index != tail_index {
            // fix up next
            {
                let next = match &mut self.contents[next_index.unwrap()] {
                    Entry::Free { .. } => panic!("Corrupted list"),
                    Entry::Occupied(e) => e,
                };

                next.prev = prev_index;
            }

            // fix up prev
            {
                let prev = match &mut self.contents[prev_index.unwrap()] {
                    Entry::Free { .. } => panic!("Corrupted list"),
                    Entry::Occupied(e) => e,
                };
                prev.next = next_index;
            }
        }

        match removed {
            Entry::Free { .. } => panic!("Corrupted list"),
            Entry::Occupied(e) => Some(e.item),
        }
    }

    /// Returns an iterator of references to the items in the list.
    ///
    /// # Examples
    ///
    /// Using an iterator to print out all of the items in a list:
    ///
    /// ```
    /// extern crate indexlist;
    ///
    /// use indexlist::IndexList;
    ///
    /// let mut list = IndexList::new();
    ///
    /// list.push_back(5);
    /// list.push_back(10);
    /// list.push_back(15);
    ///
    /// // This prints 5, 10, and then 15, each on its own line
    /// for element in list.iter() {
    ///     println!("{}", element);
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        Iter {
            list: self,
            next_index: self.head,
        }
    }

    /// Returns an `Index` to this item.
    ///
    /// If this item is not in the list, returns `None`.
    ///
    /// # Examples
    ///
    /// Finding an item:
    ///
    /// ```
    /// extern crate indexlist;
    ///
    /// use indexlist::IndexList;
    ///
    /// let mut list = IndexList::new();
    ///
    /// let five = list.push_back(5);
    ///
    /// let index = list.index_of(&5);
    ///
    /// assert_eq!(Some(five), index);
    /// ```
    pub fn index_of(&self, item: &T) -> Option<Index> {
        let generation = self.generation;

        for (index, e) in self.contents.iter().enumerate() {
            if let Entry::Occupied(e) = e {
                if &e.item == item {
                    return Some(Index { index, generation });
                }
            }
        }

        None
    }

    /// Removes the head of the list.
    ///
    /// If an item was removed, this will also return it.
    ///
    /// If this list is empty, returns `None`.
    ///
    /// # Examples
    ///
    /// Removing the head:
    ///
    /// ```
    /// extern crate indexlist;
    ///
    /// use indexlist::IndexList;
    ///
    /// let mut list = IndexList::new();
    ///
    /// list.push_back(5);
    ///
    /// assert_eq!(list.pop_front(), Some(5));
    ///
    /// assert_eq!(list.iter().count(), 0);
    /// ```
    pub fn pop_front(&mut self) -> Option<T> {
        // if we have no head, then we have an empty list, so return
        let head_index = self.head?;

        // we want to do just get, but then we run into borrowing issues.
        //
        // we could implement Entry, but... ugh. So let's fetch just the indexes for now.
        let (head_index, next_index) = match self.contents.get(head_index)? {
            Entry::Free { .. } => return None,
            Entry::Occupied(e) => (head_index, e.next),
        };

        let removed = std::mem::replace(
            // we know this index is valid thanks to the get on line 224
            &mut self.contents[head_index],
            Entry::Free {
                next_free: self.next_free,
            },
        );

        // update our free list to point to this new space
        self.next_free = Some(head_index);

        // when we remove a node, we need to increase the generation to invalidate
        // older indexes that may be refering to this spot
        self.generation += 1;

        // now we need to fix up any next or previous nodes. we have two cases:
        //
        // * index is at the head and tail (only item in the list)
        // * index is at the head

        // index is at the head and tail (only item in the list)
        if Some(head_index) == self.tail {
            self.head = None;
            self.tail = None;

        // index is at the head
        } else {
            let next = match &mut self.contents[next_index.unwrap()] {
                Entry::Free { .. } => panic!("Corrupted list"),
                Entry::Occupied(e) => e,
            };

            next.prev = None;
            self.head = next_index;
        }

        match removed {
            Entry::Free { .. } => panic!("Corrupted list"),
            Entry::Occupied(e) => Some(e.item),
        }
    }
}

struct Iter<'a, T>
where
    T: 'a,
{
    list: &'a IndexList<T>,
    next_index: Option<usize>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // do we have a next thing?
        let next_index = self.next_index?;

        // what is it?
        match &self.list.contents[next_index] {
            Entry::Free { .. } => panic!("Corrupted list"),
            Entry::Occupied(e) => {
                // set up our next iteration
                self.next_index = e.next;

                Some(&e.item)
            }
        }
    }
}

impl<T> std::ops::Index<Index> for IndexList<T>
where
    T: PartialEq,
    T: std::fmt::Debug,
{
    type Output = T;

    fn index(&self, index: Index) -> &Self::Output {
        self.get(index).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_list() {
        let _list: IndexList<i32> = IndexList::new();
    }

    #[test]
    fn insert() {
        let mut list = IndexList::new();

        list.push_back(5);

        assert_eq!(
            list.contents[0],
            Entry::Occupied(OccupiedEntry {
                item: 5,
                next: None,
                prev: None,
                generation: 0,
            })
        );
    }

    #[test]
    fn contains() {
        let mut list = IndexList::new();

        list.push_back(5);

        assert!(list.contains(&5));
    }

    #[test]
    fn get() {
        let mut list = IndexList::new();

        let five = list.push_back(5);

        let entry = list.get(five);

        assert!(entry.is_some());

        assert_eq!(entry.unwrap(), &5);
    }

    #[test]
    fn index() {
        let mut list = IndexList::new();

        let five = list.push_back(5);

        let entry = list[five];

        assert_eq!(entry, 5);
    }

    #[test]
    fn insert_thrice() {
        let mut list = IndexList::new();

        list.push_back(5);
        list.push_back(10);
        list.push_back(15);

        assert_eq!(
            list.contents[0],
            Entry::Occupied(OccupiedEntry {
                item: 5,
                next: Some(1),
                prev: None,
                generation: 0,
            })
        );

        assert_eq!(
            list.contents[1],
            Entry::Occupied(OccupiedEntry {
                item: 10,
                next: Some(2),
                prev: Some(0),
                generation: 0,
            })
        );

        assert_eq!(
            list.contents[2],
            Entry::Occupied(OccupiedEntry {
                item: 15,
                next: None,
                prev: Some(1),
                generation: 0,
            })
        );
    }

    #[test]
    fn remove_middle() {
        let mut list = IndexList::new();

        list.push_back(5);
        let ten = list.push_back(10);
        list.push_back(15);

        let removed = list.remove(ten).unwrap();

        assert_eq!(removed, 10);

        assert_eq!(
            list,
            IndexList {
                contents: vec![
                    Entry::Occupied(OccupiedEntry {
                        item: 5,
                        next: Some(2),
                        prev: None,
                        generation: 0,
                    }),
                    Entry::Free { next_free: None },
                    Entry::Occupied(OccupiedEntry {
                        item: 15,
                        next: None,
                        prev: Some(0),
                        generation: 0,
                    }),
                ],
                generation: 1,
                next_free: Some(1),
                head: Some(0),
                tail: Some(2),
            }
        );
    }

    #[test]
    fn remove_head() {
        let mut list = IndexList::new();

        let five = list.push_back(5);
        list.push_back(10);
        list.push_back(15);

        let removed = list.remove(five).unwrap();

        assert_eq!(removed, 5);

        assert_eq!(
            list,
            IndexList {
                contents: vec![
                    Entry::Free { next_free: None },
                    Entry::Occupied(OccupiedEntry {
                        item: 10,
                        next: Some(2),
                        prev: None,
                        generation: 0,
                    }),
                    Entry::Occupied(OccupiedEntry {
                        item: 15,
                        next: None,
                        prev: Some(1),
                        generation: 0,
                    }),
                ],
                generation: 1,
                next_free: Some(0),
                head: Some(1),
                tail: Some(2),
            }
        );
    }

    #[test]
    fn remove_tail() {
        let mut list = IndexList::new();

        list.push_back(5);
        list.push_back(10);
        let fifteen = list.push_back(15);

        let removed = list.remove(fifteen).unwrap();

        assert_eq!(removed, 15);

        assert_eq!(
            list,
            IndexList {
                contents: vec![
                    Entry::Occupied(OccupiedEntry {
                        item: 5,
                        next: Some(1),
                        prev: None,
                        generation: 0,
                    }),
                    Entry::Occupied(OccupiedEntry {
                        item: 10,
                        next: None,
                        prev: Some(0),
                        generation: 0,
                    }),
                    Entry::Free { next_free: None },
                ],
                generation: 1,
                next_free: Some(2),
                head: Some(0),
                tail: Some(1),
            }
        );
    }

    #[test]
    fn remove_only() {
        let mut list = IndexList::new();

        let five = list.push_back(5);

        let removed = list.remove(five).unwrap();

        assert_eq!(removed, 5);

        assert_eq!(
            list,
            IndexList {
                contents: vec![Entry::Free { next_free: None },],
                generation: 1,
                next_free: Some(0),
                head: None,
                tail: None,
            }
        );
    }

    #[test]
    fn remove_returns_none_when_not_there() {
        let mut list = IndexList::new();

        let five_index = list.push_back(5);

        let five_entry = list.remove(five_index).unwrap();

        assert_eq!(list.contents[0], Entry::Free { next_free: None });

        assert_eq!(five_entry, 5);

        assert!(list.remove(five_index).is_none());
    }

    #[test]
    fn iter() {
        let mut list = IndexList::new();

        list.push_back(5);
        let ten = list.push_back(10);
        list.push_back(15);

        list.remove(ten);

        let mut iter = list.iter();

        assert_eq!(iter.next().unwrap(), &5);
        assert_eq!(iter.next().unwrap(), &15);

        assert!(iter.next().is_none());
    }

    #[test]
    fn reallocation() {
        let mut list = IndexList::new();

        list.push_back(5);
        let ten = list.push_back(10);
        list.push_back(15);

        let ten = list.remove(ten).unwrap();

        assert_eq!(ten, 10);

        list.push_back(20);

        assert_eq!(
            list.contents[0],
            Entry::Occupied(OccupiedEntry {
                item: 5,
                next: Some(2),
                prev: None,
                generation: 0,
            })
        );

        assert_eq!(
            list.contents[1],
            Entry::Occupied(OccupiedEntry {
                item: 20,
                next: None,
                prev: Some(2),
                generation: 1,
            })
        );

        assert_eq!(
            list.contents[2],
            Entry::Occupied(OccupiedEntry {
                item: 15,
                next: Some(1),
                prev: Some(0),
                generation: 0,
            })
        );
    }

    #[test]
    fn generations() {
        let mut list = IndexList::new();

        let five = list.push_back(5);
        let ten = list.push_back(10);
        list.push_back(15);

        list.remove(ten);

        let twenty = list.push_back(20);

        // since we reallocate, that twenty should have gone where the ten was.
        // this means that ten should now be invalid.
        assert!(list.get(ten).is_none());

        // however, five should be fine!
        assert!(list.get(five).is_some());

        // as should twenty!
        assert!(list.get(twenty).is_some());
    }

    #[test]
    fn head() {
        let mut list = IndexList::new();

        assert!(list.head().is_none());

        let five = list.push_back(5);

        assert_eq!(list.head().unwrap(), &5);

        list.push_back(10);

        list.remove(five);

        assert_eq!(list.head().unwrap(), &10);

        assert_eq!(list.contents[0], Entry::Free { next_free: None });

        assert_eq!(list.head, Some(1));

        assert_eq!(
            list.contents[1],
            Entry::Occupied(OccupiedEntry {
                item: 10,
                next: None,
                prev: None,
                generation: 0,
            })
        );
    }

    #[test]
    fn push_front() {
        let mut list = IndexList::new();

        list.push_front(5);
        list.push_front(10);
        list.push_front(15);

        assert_eq!(
            list.contents[0],
            Entry::Occupied(OccupiedEntry {
                item: 5,
                next: None,
                prev: Some(1),
                generation: 0,
            })
        );

        assert_eq!(
            list.contents[1],
            Entry::Occupied(OccupiedEntry {
                item: 10,
                next: Some(0),
                prev: Some(2),
                generation: 0,
            })
        );

        assert_eq!(
            list.contents[2],
            Entry::Occupied(OccupiedEntry {
                item: 15,
                next: Some(1),
                prev: None,
                generation: 0,
            })
        );
    }

    #[test]
    fn index_of() {
        let mut list = IndexList::new();

        list.push_back(5);
        list.push_back(10);
        list.push_back(15);

        assert_eq!(
            list.index_of(&10).unwrap(),
            Index {
                index: 1,
                generation: 0,
            }
        );

        assert!(list.index_of(&20).is_none());
    }

    #[test]
    fn pop_front() {
        let mut list = IndexList::new();

        list.push_back(5);
        list.push_back(10);
        list.push_back(15);

        assert_eq!(list.pop_front().unwrap(), 5);
        assert_eq!(list.pop_front().unwrap(), 10);
        assert_eq!(list.pop_front().unwrap(), 15);

        assert_eq!(
            list,
            IndexList {
                contents: vec![
                    Entry::Free { next_free: None },
                    Entry::Free { next_free: Some(0) },
                    Entry::Free { next_free: Some(1) },
                ],
                generation: 3,
                next_free: Some(2),
                head: None,
                tail: None,
            }
        );
    }

    #[test]
    fn push_and_pop() {
        let mut list = IndexList::new();

        list.push_back(5);
        list.push_back(10);
        list.push_back(15);

        assert_eq!(list.pop_front().unwrap(), 5);
        assert_eq!(list.pop_front().unwrap(), 10);
        assert_eq!(list.pop_front().unwrap(), 15);

        list.push_back(5);
        list.push_back(10);
        list.push_back(15);

        assert_eq!(list.pop_front().unwrap(), 5);
        assert_eq!(list.pop_front().unwrap(), 10);
        assert_eq!(list.pop_front().unwrap(), 15);

        assert_eq!(
            list,
            IndexList {
                contents: vec![
                    Entry::Free { next_free: Some(1) },
                    Entry::Free { next_free: Some(2) },
                    Entry::Free { next_free: None },
                ],
                generation: 6,
                next_free: Some(0),
                head: None,
                tail: None,
            }
        );
    }
}
