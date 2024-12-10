use std::ptr::NonNull;

use super::data::Data;

pub(crate) struct Node<K, V> {
    pub(crate) data: Option<Data<K, V>>,
    pub(crate) next: Option<Box<Node<K, V>>>,
    pub(crate) forward: Vec<Option<NonNull<Node<K, V>>>>,
}

impl<K: Ord, V> Node<K, V> {
    #[inline]
    pub(crate) fn new(data: Option<Data<K, V>>, level: usize) -> Self {
        Self {
            data,
            next: None,
            forward: vec![None; level],
        }
    }

    #[inline]
    pub(crate) fn with_key_value(key: K, value: V, level: usize) -> Self {
        Self::new(Some((key, value).into()), level)
    }

    #[inline]
    pub(crate) fn level(&self) -> usize {
        self.forward.len()
    }

    #[inline]
    pub(crate) fn key(&self) -> Option<&K> {
        self.data.as_ref().map(|data| &data.key)
    }

    #[inline]
    pub(crate) fn value(&self) -> Option<&V> {
        self.data.as_ref().map(|data| &data.value)
    }

    #[inline]
    pub(crate) fn value_mut(&mut self) -> Option<&mut V> {
        self.data.as_mut().map(|data| &mut data.value)
    }

    #[inline]
    pub(crate) fn exchange_value(&mut self, value: V) -> Option<V> {
        self.data
            .as_mut()
            .map(|data| std::mem::replace(&mut data.value, value))
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn next_node_ptr(&self) -> Option<NonNull<Node<K, V>>> {
        self.next.as_ref().map(|node| NonNull::from(node.as_ref()))
    }

    #[inline]
    pub(crate) fn connect_next_node(&mut self, mut next: Box<Node<K, V>>) {
        next.as_mut().next = self.next.take();
        self.next.replace(next);
    }
}

impl<K, V> Into<Option<Data<K, V>>> for Node<K, V> {
    fn into(self) -> Option<Data<K, V>> {
        self.data
    }
}

impl<K, V> Into<Option<(K, V)>> for Node<K, V> {
    fn into(self) -> Option<(K, V)> {
        self.data.map(|data| data.into())
    }
}

// impl<K, V> Drop for Node<K, V> {
//     fn drop(&mut self) {
//         let mut node = match self.next.take() {
//             Some(node) => node,
//             None => return,
//         };

//         while let Some(next_node) = node.next.take() {
//             drop(node);
//             node = next_node;
//         }
//     }
// }
