use std::collections::HashMap;

use itertools::Itertools;
use rand::random;
use rust_ds::collections::skip_list::SkipList;

fn main() {
    const TEST_SIZE: usize = 10000000;

    let mut data: HashMap<u64, u64> = HashMap::new();
    while data.len() < TEST_SIZE {
        let k = random();
        if data.contains_key(&k) {
            continue;
        }
        let v = random();
        data.insert(k, v);
    }

    let data = data.into_iter().sorted().collect_vec();

    let mut skip_list = SkipList::new();
    for &(k, v) in data.iter() {
        skip_list.insert(k, v);
    }

    let skip_list2 = skip_list.clone();

    assert_eq!(skip_list.len(), skip_list2.len());

    for (k, v) in skip_list.iter() {
        assert_eq!(skip_list2.get(k), Some(v));
    }

    assert_eq!(
        skip_list.into_iter().collect_vec(),
        skip_list2.into_iter().collect_vec()
    );

    println!("Test passed.");
}
