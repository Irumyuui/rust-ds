use super::{Node, SkipList};

pub struct Iter<'a, K: 'a, V: 'a> {
    current: Option<&'a Node<K, V>>,
    _marker: std::marker::PhantomData<&'a Node<K, V>>,
}

impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.take() {
            Some(node) => {
                self.current = match node.next.as_ref() {
                    Some(node) => Some(node.as_ref()),
                    None => None,
                };

                let result = node
                    .data
                    .as_ref()
                    .map(|data| (&data.key, &data.value))
                    .expect("must have data.");

                Some(result)
            }
            None => None,
        }
    }
}

pub struct IterMut<'a, K: 'a, V: 'a> {
    current: Option<&'a mut Node<K, V>>,
    _marker: std::marker::PhantomData<&'a mut Node<K, V>>,
}

impl<'a, K: 'a, V: 'a> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.take() {
            Some(node) => {
                self.current = node.next.as_mut().map(|node| node.as_mut());

                let result = node
                    .data
                    .as_mut()
                    .map(|data| (&data.key, &mut data.value))
                    .expect("must have data.");

                Some(result)
            }
            None => todo!(),
        }
    }
}

pub struct IntoIter<K: Ord, V> {
    inner: SkipList<K, V>,
}

impl<K: Ord, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.remove_front().map(|data| data.into())
    }
}

impl<K: Ord, V> IntoIterator for SkipList<K, V> {
    type Item = (K, V);

    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { inner: self }
    }
}

impl<K: Ord, V> SkipList<K, V> {
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            current: self.head.next.as_ref().map(|node| node.as_ref()),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            current: self.head.next.as_mut().map(|node| node.as_mut()),
            _marker: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::collections::skip_list::SkipList;

    #[test]
    fn iter() {
        const TEST_CASE: usize = 10000000;

        let mut sl = SkipList::new();

        for i in 0..TEST_CASE {
            let res = sl.insert(i, i.to_string());
            assert!(res.is_none());
        }

        assert_eq!(sl.len(), TEST_CASE);

        for (i, (k, v)) in sl.iter().enumerate() {
            assert_eq!(*k, i);
            assert_eq!(*v, i.to_string());
        }

        for i in 0..TEST_CASE {
            let res = sl.insert(i, (i * 10).to_string());
            assert!(res.is_some_and(|s| s == i.to_string()));
        }

        assert_eq!(sl.len(), TEST_CASE);

        for (i, (k, v)) in sl.iter().enumerate() {
            assert_eq!(*k, i);
            assert_eq!(*v, (i * 10).to_string());
        }
    }
}
