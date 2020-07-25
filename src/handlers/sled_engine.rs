use std::convert::TryInto;

use sled::{Config, Db, Tree};

pub struct SledEngine {
    path: String,
    db: Db,
}

impl SledEngine {
    pub fn new(path: &str) -> Self {
        SledEngine {
            path: path.to_owned(),
            db: sled::open(&path).unwrap(),
        }
    }

    pub fn new_tmp() -> Self {
        SledEngine {
            path: "".to_owned(),
            db: Config::new().temporary(true).open().unwrap(),
        }
    }

    pub fn open_tree(&self, name: &str) -> Tree {
        self.db.open_tree(name).unwrap()
    }

    pub fn drop_tree(&self, name: &str) {
        self.db.drop_tree(name);
    }

    pub fn get(&self, tree_name: &str, key: &[u8]) -> Option<Vec<u8>> {
        match self.open_tree(tree_name).get(key).unwrap() {
            Some(res) => Some(res.to_vec()),
            None => None,
        }
    }

    pub fn insert(&self, tree_name: &str, key: &[u8], value: &[u8]) {
        self.open_tree(tree_name).insert(key, value);
    }

    pub fn remove(&self, tree_name: &str, key: &[u8]) {
        self.open_tree(tree_name).remove(key);
    }

    pub fn increment(&self, tree_name: &str, key: &str) -> u64 {
        let tree = self.open_tree(tree_name);
        Self::bytes_to_long(
            &tree
                .update_and_fetch(key.as_bytes(), |old| -> Option<Vec<u8>> {
                    let number = match old {
                        Some(bytes) => {
                            let number = Self::bytes_to_long(bytes);
                            number + 1
                        }
                        None => 0,
                    };
                    Some(number.to_be_bytes().to_vec())
                })
                .unwrap()
                .unwrap()
                .to_vec()
                .to_owned(),
        )
    }

    fn bytes_to_long(bytes: &[u8]) -> u64 {
        let array: [u8; 8] = bytes.try_into().unwrap();
        u64::from_be_bytes(array)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sled_crud() {
        let sled = SledEngine::new_tmp();
        let tree1 = "test0";
        let tree2 = "test1";
        let key1 = "key1".as_bytes();
        let key2 = "key2".as_bytes();
        let value1 = "v1".as_bytes();
        let value2 = "v2".as_bytes();

        sled.insert(tree1, key1, value1);
        assert_eq!(sled.get(tree1, key1).unwrap(), value1);
        sled.insert(tree1, key1, value2);
        assert_eq!(sled.get(tree1, key1).unwrap(), value2);

        sled.insert(tree2, key1, value1);
        assert_eq!(sled.get(tree2, key1).unwrap(), value1);
        sled.insert(tree2, key1, value2);
        assert_eq!(sled.get(tree2, key1).unwrap(), value2);

        sled.remove(tree1, key1);
        assert_eq!(sled.get(tree1, key1), None);
        sled.remove(tree2, key1);
        assert_eq!(sled.get(tree2, key1), None);
    }

    #[test]
    fn test_increment() {
        let sled = SledEngine::new_tmp();
        let i = sled.increment("tree1", "11");
        assert_eq!(i, 0);
        let i = sled.increment("tree1", "11");
        assert_eq!(i, 1);
        let i = sled.increment("tree1", "11");
        assert_eq!(i, 2);
    }
}
