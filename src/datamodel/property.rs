use crate::datamodel::buffer::Buffer;
use crate::datamodel::constants::STRING_TERM;

#[derive(Debug, Eq, PartialEq)]
pub struct Properties {
    pub(crate) data: Vec<u8>
}

#[derive(Debug, Eq, PartialEq)]
pub struct Property {
    key: u64,
    id: u64,
    pub(crate) value: String,
}

impl Properties {
    pub fn get_properties(&self) -> Box<PropertyIterator> {
        let mut properties = Buffer::from(&self.data);
        Box::new(PropertyIterator {
            properties,
            offset: 0,
            predicate: Box::new(|_, _, _| { true }),
        })
    }

    pub fn get_property(&self, key_id: u64) -> Box<PropertyIterator> {
        let mut properties = Buffer::from(&self.data);
        Box::new(PropertyIterator {
            properties,
            offset: 0,
            predicate: Box::new(move |key, _, _| { key == key_id }),
        })
    }

    pub fn add_property(&mut self, key: u64, prop_id: u64, value: &str) {
        let mut property = Buffer::new();
        property.put_u64(key);
        property.put_u64(value.len() as u64);
        property.put_u64(prop_id);
        property.put_string(value);

        self.data.append(&mut property.bytes().to_vec());
    }

    pub fn remove_property(&mut self, key: u64, prop_id: Vec<u64>) {
        let mut data = Buffer::from(&self.data);
        let mut new_data = Buffer::new();
        while data.has_remaining() {
            let key_id = data.get_u64();
            let value_len = data.get_u64();
            let pid = data.get_u64();
            let value = data.get_string_raw();
            if key == key_id && (prop_id.is_empty() || prop_id.contains(&pid)) {
                continue;
            }
            new_data.put_u64(key_id);
            new_data.put_u64(value_len);
            new_data.put_u64(pid);
            new_data.put_slice(&value);
            new_data.put_u8(STRING_TERM);
        }
        self.data = new_data.to_vec();
    }
}


pub struct PropertyIterator {
    properties: Buffer,
    offset: u32,
    predicate: Box<dyn Fn(u64, u64, &str) -> bool>,
}

impl Iterator for PropertyIterator {
    type Item = Property;

    fn next(&mut self) -> Option<Self::Item> {
        while self.properties.has_remaining() {
            let key_id = self.properties.get_u64();
            let value_len = self.properties.get_u64() as usize;
            let prop_id = self.properties.get_u64();
            let value = self.properties.get_string_utf8();

            if (self.predicate)(key_id, prop_id, &value) {
                return Some(Property {
                    key: key_id,
                    id: prop_id,
                    value,
                });
            }
        }
        None
    }
}


#[cfg(test)]
mod test {
    use super::*;

    fn build_test_properties() -> Properties {
        let mut properties = Properties { data: Vec::new() };
        properties.add_property(12, 99, "hello angelina");
        properties.add_property(13, 100, "hello angelina2");
        properties
    }

    #[test]
    fn test_get_properties() {
        let properties = build_test_properties();
        for p in properties.get_properties() {
            println!("{:?}", p)
        }
    }

    #[test]
    fn test_get_property() {
        let properties = build_test_properties();

        for p in properties.get_property(12) {
            assert_eq!(p, Property { key: 12, id: 99, value: "hello angelina".to_string() })
        }
        for p in properties.get_property(13) {
            assert_eq!(p, Property { key: 13, id: 100, value: "hello angelina2".to_string() })
        }
    }

    #[test]
    fn test_write_property() {
        let mut properties = build_test_properties();
        properties.add_property(14, 101, "hello angelina3");
        for p in properties.get_property(14) {
            assert_eq!(p, Property { key: 14, id: 101, value: "hello angelina3".to_string() })
        }
    }

    #[test]
    fn test_multi_property() {
        let mut properties = build_test_properties();
        properties.add_property(12, 102, "hello angelina3");
        properties.add_property(12, 104, "hello angelina3");
        assert_eq!(
            properties.get_property(12).map(|p| { p.id }).collect::<Vec<u64>>(),
            vec![99, 102, 104])
    }
}
