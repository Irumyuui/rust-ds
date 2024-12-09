use std::ptr::NonNull;

const MAX_LEVEL: usize = 32;
const P: f64 = 0.5;

struct Data<K, V> {
    key: K,
    value: V,
}

struct Node<K, V> {
    data: Option<Data<K, V>>,
    forward: Vec<Option<NonNull<Node<K, V>>>>,
}

impl<K, V> Node<K, V> {
    fn new(data: Option<Data<K, V>>, level: usize) -> Self {
        Self {
            data,
            forward: vec![None; level + 1],
        }
    }

    fn head() -> Self {
        Self::new(None, MAX_LEVEL - 1)
    }

    #[allow(unused)]
    fn level(&self) -> usize {
        self.forward.len() - 1
    }
}

fn random_level() -> usize {
    let mut level = 0;
    let mut x = P;
    let f = 1.0 - rand::random::<f64>();

    while x > f && level + 1 < MAX_LEVEL {
        level += 1;
        x *= P;
    }

    level
}

pub struct SkipList<K, V> {
    len: usize,
    head: NonNull<Node<K, V>>,
}

impl<K, V> SkipList<K, V>
where
    K: Ord,
{
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            len: 0,
            head: NonNull::new(Box::into_raw(Box::new(Node::head())))
                .expect("Failed to allocate node memory."),
        }
    }

    #[allow(unused)]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let mut update_nodes: [NonNull<Node<K, V>>; MAX_LEVEL] = [self.head; MAX_LEVEL];

        let mut cur_node = self.head;
        for i in (0..MAX_LEVEL).rev() {
            while let Some(&next_node) = unsafe { cur_node.as_ref().forward[i].as_ref() } {
                if unsafe {
                    next_node
                        .as_ref()
                        .data
                        .as_ref()
                        .is_some_and(|d| d.key < key)
                } {
                    cur_node = next_node;
                } else {
                    break;
                }
            }

            update_nodes[i] = cur_node;
        }

        if cfg!(debug_assertions) {
            assert!(update_nodes
                .iter()
                .all(|ptr| { ptr.as_ptr() != std::ptr::null_mut() }));
        }

        if let Some(next_node) = unsafe { cur_node.as_mut().forward[0].as_mut() } {
            if let Some(data) = unsafe { next_node.as_mut().data.as_mut() } {
                if data.key == key {
                    let value = std::mem::replace(&mut data.value, value);
                    return Some(value);
                }
            }
        }

        let new_level = random_level();

        if cfg!(debug_assertions) {
            assert!(new_level <= MAX_LEVEL);
        }

        let mut new_node = NonNull::new(Box::into_raw(Box::new(Node::new(
            Some(Data { key, value }),
            new_level,
        ))))
        .expect("Failed to allocate node memory on insert.");

        for i in 0..=new_level {
            let mut node = update_nodes[i];
            unsafe {
                new_node.as_mut().forward[i] = node.as_ref().forward[i];
                node.as_mut().forward[i] = Some(new_node);
            }
        }

        self.len += 1;

        None
    }

    #[allow(unused)]
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let mut update_nodes: [NonNull<Node<K, V>>; MAX_LEVEL] = [self.head; MAX_LEVEL];

        let mut cur_node = self.head;
        for i in (0..MAX_LEVEL).rev() {
            while let Some(next_node) = unsafe { cur_node.as_ref().forward[i] } {
                if unsafe {
                    next_node
                        .as_ref()
                        .data
                        .as_ref()
                        .is_some_and(|d| d.key < *key)
                } {
                    cur_node = next_node;
                } else {
                    break;
                }
            }

            update_nodes[i] = cur_node;
        }

        if cfg!(debug_assertions) {
            assert!(update_nodes
                .iter()
                .all(|ptr| { ptr.as_ptr() != std::ptr::null_mut() }));
        }

        let del_node = unsafe {
            if cur_node.as_ref().forward[0]
                .as_ref()
                .is_some_and(|next_node| {
                    next_node
                        .as_ref()
                        .data
                        .as_ref()
                        .is_some_and(|d| d.key == *key)
                })
            {
                cur_node.as_mut().forward[0].unwrap()
            } else {
                return None;
            }
        };

        for i in 0..MAX_LEVEL {
            let mut node = update_nodes[i];
            if unsafe { node.as_ref().forward[i].is_none_or(|ptr| ptr != del_node) } {
                break;
            }

            unsafe {
                node.as_mut().forward[i] = del_node.as_ref().forward[i];
            }
        }

        let node = unsafe { Box::from_raw(del_node.as_ptr()) };
        self.len -= 1;
        Some(node.data.unwrap().value)
    }

    #[allow(unused)]
    pub fn get(&self, key: &K) -> Option<&V> {
        let mut cur_node = self.head;
        for i in (0..MAX_LEVEL).rev() {
            while let Some(next_node) = unsafe { cur_node.as_ref().forward[i] } {
                if unsafe {
                    next_node
                        .as_ref()
                        .data
                        .as_ref()
                        .is_some_and(|d| d.key < *key)
                } {
                    cur_node = next_node;
                } else {
                    break;
                }
            }
        }

        if let Some(next_node) = unsafe { cur_node.as_ref().forward[0] } {
            if let Some(data) = unsafe { next_node.as_ref().data.as_ref() } {
                if data.key == *key {
                    return Some(&data.value);
                }
            }
        }

        None
    }

    #[allow(unused)]
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let mut cur_node = self.head;
        for i in (0..MAX_LEVEL).rev() {
            while let Some(next_node) = unsafe { cur_node.as_ref().forward[i] } {
                if unsafe {
                    next_node
                        .as_ref()
                        .data
                        .as_ref()
                        .is_some_and(|d| d.key < *key)
                } {
                    cur_node = next_node;
                } else {
                    break;
                }
            }
        }

        if let Some(mut next_node) = unsafe { cur_node.as_mut().forward[0] } {
            if let Some(data) = unsafe { next_node.as_mut().data.as_mut() } {
                if data.key == *key {
                    return Some(&mut data.value);
                }
            }
        }

        None
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl<K, V> Drop for SkipList<K, V> {
    fn drop(&mut self) {
        let mut node = self.head;
        unsafe {
            while let Some(next_node) = node.as_ref().forward[0] {
                let _ = Box::from_raw(node.as_ptr());
                node = next_node;
            }

            let _ = Box::from_raw(node.as_ptr());
        }
    }
}

impl<K, V> std::fmt::Display for Data<K, V>
where
    K: std::fmt::Display,
    V: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.key, self.value)
    }
}

impl<K, V> std::fmt::Display for SkipList<K, V>
where
    K: std::fmt::Display,
    V: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;

        let mut node = self.head;
        if let Some(next_node) = unsafe { node.as_ref().forward[0] } {
            if let Some(data) = unsafe { next_node.as_ref().data.as_ref() } {
                write!(f, "{}", data)?;
            }
            node = next_node;
        }

        while let Some(next_node) = unsafe { node.as_ref().forward[0] } {
            if let Some(data) = unsafe { next_node.as_ref().data.as_ref() } {
                write!(f, ", {}", data)?;
            }
            node = next_node;
        }

        write!(f, "}}")
    }
}
