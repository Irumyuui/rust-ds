use std::fmt::Display;

pub(crate) struct Data<K, V> {
    pub(crate) key: K,
    pub(crate) value: V,
}

impl<K, V> From<(K, V)> for Data<K, V> {
    fn from(value: (K, V)) -> Self {
        Data {
            key: value.0,
            value: value.1,
        }
    }
}

impl<K, V> From<Data<K, V>> for (K, V) {
    fn from(value: Data<K, V>) -> Self {
        (value.key, value.value)
    }
}

impl<K: Display, V: Display> Display for Data<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.key, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::Data;

    #[test]
    fn test_data() {
        let data: Data<_, _> = (1, "hello").into();
        assert_eq!(format!("{}", data), "(1, hello)");
        let data: (_, _) = data.into();
        assert_eq!(data, (1, "hello"));
    }
}
