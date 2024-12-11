use std::{fmt::Display, ptr::NonNull};

use data::Data;
use node::Node;

mod data;
mod gen_level;
mod iter;
mod node;

pub use gen_level::*;
pub use iter::*;

pub const MAX_LEVEL: usize = 32;

pub struct SkipList<K, V, G = DefaultGenerator>
where
    K: Ord,
    G: LevelGenerator,
{
    head: Node<K, V>,
    len: usize,
    gen: G,
}

impl<K, V> SkipList<K, V, DefaultGenerator>
where
    K: Ord,
{
    pub fn new() -> SkipList<K, V, DefaultGenerator> {
        Default::default()
    }
}

impl<K, V, G> SkipList<K, V, G>
where
    K: Ord,
    G: LevelGenerator,
{
    pub fn with_gen(gen: G) -> Self {
        Self {
            head: Node::new(None, MAX_LEVEL),
            len: 0,
            gen,
        }
    }

    fn get_adjust_nodes(
        head: &Node<K, V>,
        key: &K,
    ) -> (
        NonNull<Node<K, V>>,
        [Option<NonNull<Node<K, V>>>; MAX_LEVEL],
    ) {
        let mut adjust_nodes: [Option<NonNull<Node<K, V>>>; MAX_LEVEL] = [None; MAX_LEVEL];

        let mut cur_ptr = NonNull::from(head);
        for find_level in (0..MAX_LEVEL).rev() {
            unsafe {
                while let Some(next_ptr) = cur_ptr.as_ref().forward[find_level] {
                    if next_ptr.as_ref().key().is_some_and(|k| k < key) {
                        cur_ptr = next_ptr;
                    } else {
                        break;
                    }
                }
            }

            adjust_nodes[find_level] = Some(cur_ptr);
        }

        (cur_ptr, adjust_nodes)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let (mut cur_ptr, adjust_nodes) = Self::get_adjust_nodes(&self.head, &key);

        if let Some(next_node) = unsafe { cur_ptr.as_mut().next.as_mut() } {
            if next_node.as_ref().key().is_some_and(|k| key == *k) {
                return next_node.as_mut().exchange_value(value);
            }
        }

        let new_level = self.gen.random_level();
        let new_node = Node::with_key_value(key, value, new_level);
        let mut new_node = Box::new(new_node);
        let new_node_ptr = NonNull::from(new_node.as_ref());

        for adjust_level in 0..new_node.level() {
            let mut adj_node = unsafe { adjust_nodes.get_unchecked(adjust_level).unwrap() };
            unsafe {
                new_node.as_mut().forward[adjust_level] = adj_node.as_ref().forward[adjust_level];
                adj_node.as_mut().forward[adjust_level].replace(new_node_ptr);
            }
        }

        unsafe {
            cur_ptr.as_mut().connect_next_node(new_node);
        }

        self.len += 1;

        None
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let (mut cur_ptr, adjust_nodes) = Self::get_adjust_nodes(&self.head, key);

        let mut del_node_ptr = None;
        if let Some(next_node) = unsafe { cur_ptr.as_ref().next.as_ref() } {
            if next_node.as_ref().key().is_some_and(|k| key == k) {
                del_node_ptr = Some(NonNull::from(next_node.as_ref()));
            }
        }

        let del_node_ptr = match del_node_ptr {
            Some(ptr) => ptr,
            None => return None,
        };

        for (i, &node_ptr) in adjust_nodes.iter().enumerate() {
            let mut node_ptr = node_ptr.unwrap();
            unsafe {
                if node_ptr.as_ref().forward[i].is_some_and(|p| p != del_node_ptr) {
                    break;
                }

                node_ptr.as_mut().forward[i] = match del_node_ptr.as_ref().forward.get(i) {
                    Some(ptr) => ptr.clone(),
                    None => None,
                };
            }
        }

        let data = unsafe {
            let mut del_node = match cur_ptr.as_mut().next.take() {
                Some(node) => node,
                None => unreachable!("Must have a next node."),
            };
            cur_ptr.as_mut().next = del_node.as_mut().next.take();

            del_node.data.take()
        };

        self.len -= 1;
        data.map(|d| d.value)
    }

    fn get_node_ptr(head: &Node<K, V>, key: &K) -> Option<NonNull<Node<K, V>>> {
        let mut cur_ptr = NonNull::from(head);

        for find_level in (0..MAX_LEVEL).rev() {
            unsafe {
                while let Some(next_ptr) = cur_ptr.as_ref().forward[find_level] {
                    match next_ptr.as_ref().key() {
                        Some(k) => match k.cmp(key) {
                            std::cmp::Ordering::Less => cur_ptr = next_ptr,
                            std::cmp::Ordering::Equal => return Some(next_ptr),
                            std::cmp::Ordering::Greater => break,
                        },
                        None => break,
                    }
                }
            }
        }

        None
    }

    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        match Self::get_node_ptr(&self.head, key) {
            Some(node_ptr) => unsafe { node_ptr.as_ref().value() },
            None => None,
        }
    }

    #[inline]
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match Self::get_node_ptr(&self.head, key) {
            Some(mut node_ptr) => unsafe { node_ptr.as_mut().value_mut() },
            None => None,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn clear(&mut self) {
        while !self.is_empty() {
            self.remove_front();
        }
    }

    fn remove_front(&mut self) -> Option<Data<K, V>> {
        let mut del_node = match self.head.next.take() {
            Some(node) => node,
            None => return None,
        };

        self.head.next = del_node.as_mut().next.take();
        self.len -= 1;

        for level in 0..del_node.level() {
            self.head.forward[level] = del_node.as_ref().forward[level];
        }

        del_node.data.take()
    }
}

impl<K, V, G> Default for SkipList<K, V, G>
where
    K: Ord,
    G: LevelGenerator + Default,
{
    #[inline]
    fn default() -> SkipList<K, V, G> {
        Self::with_gen(Default::default())
    }
}

impl<K, V, G> Drop for SkipList<K, V, G>
where
    K: Ord,
    G: LevelGenerator,
{
    fn drop(&mut self) {
        let mut node = match self.head.next.take() {
            Some(node) => node,
            None => return,
        };

        while let Some(next) = node.next.take() {
            drop(node);
            node = next;
        }
    }
}

impl<K, V, G> Clone for SkipList<K, V, G>
where
    K: Ord + Clone,
    V: Clone,
    G: LevelGenerator + Clone,
{
    fn clone(&self) -> Self {
        let mut new_sl = SkipList::with_gen(self.gen.clone());
        for (key, value) in self.iter() {
            new_sl.insert(key.clone(), value.clone());
        }
        new_sl
    }
}

impl<K, V> Display for SkipList<K, V>
where
    K: Ord + Display,
    V: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut it = self.iter();

        write!(f, "{{")?;
        if let Some((key, value)) = it.next() {
            write!(f, "({}: {})", key, value)?;
        }
        for (key, value) in it {
            write!(f, ",({}: {})", key, value)?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::SkipList;

    #[test]
    fn insert_and_get() {
        let mut skiplist = SkipList::new();

        for i in 0..100000 {
            skiplist.insert(i, i);
        }
        for i in 0..100000 {
            assert_eq!(skiplist.get(&i), Some(&i));
        }
    }

    #[test]
    fn update_value() {
        let mut skiplist = SkipList::new();

        skiplist.insert(1, "value1");
        assert_eq!(skiplist.insert(1, "value2"), Some("value1"));
        assert_eq!(skiplist.get(&1), Some(&"value2"));
    }

    #[test]
    fn remove() {
        let mut skiplist = SkipList::new();

        skiplist.insert(1, "value1");
        skiplist.insert(2, "value2");
        skiplist.insert(3, "value3");

        assert_eq!(skiplist.remove(&2), Some("value2"));
        assert_eq!(skiplist.get(&2), None);
        assert_eq!(skiplist.len(), 2);
    }

    #[test]
    fn remove_non_existent() {
        let mut skiplist = SkipList::new();

        skiplist.insert(1, "value1");
        assert_eq!(skiplist.remove(&2), None);
    }

    #[test]
    fn clear() {
        let mut skiplist = SkipList::new();

        skiplist.insert(1, "value1");
        skiplist.insert(2, "value2");
        skiplist.clear();

        assert_eq!(skiplist.len(), 0);
        assert!(skiplist.is_empty());
    }

    #[test]
    fn clone() {
        let mut skiplist = SkipList::new();
        skiplist.insert(1, "value1");
        skiplist.insert(2, "value2");

        let cloned_skiplist = skiplist.clone();

        assert_eq!(cloned_skiplist.get(&1), Some(&"value1"));
        assert_eq!(cloned_skiplist.get(&2), Some(&"value2"));
        assert_eq!(cloned_skiplist.len(), 2);
    }

    #[test]
    fn display() {
        let mut skiplist = SkipList::new();
        skiplist.insert(1, "value1");
        skiplist.insert(2, "value2");

        assert_eq!(format!("{}", skiplist), "{(1: value1),(2: value2)}");

        skiplist.remove(&1);
        assert_eq!(format!("{}", skiplist), "{(2: value2)}");
    }
}
