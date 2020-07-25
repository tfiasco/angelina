use crate::datamodel::base::{BaseSchemaModel, Cardinality, SchemaType};
use crate::datamodel::buffer::Buffer;

#[derive(Debug, Eq, PartialEq)]
pub struct PropertyKey {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) cardinality: Cardinality,
}

impl BaseSchemaModel for PropertyKey {
    fn serialize(&self) -> (Vec<u8>, Vec<u8>) {
        let key = Self::build_key(self.id);
        let mut value_buf = Buffer::new();
        value_buf.put_string(&self.name);
        value_buf.put_u8(self.cardinality as u8);
        (key, value_buf.to_vec())
    }

    fn deserialize(key: &[u8], value: &[u8]) -> Self {
        let mut key_buf = Buffer::from(key);
        key_buf.get_u8(); // SchemaType
        let id = key_buf.get_u64();

        Self::deserialize_value(id, value)
    }

    fn deserialize_value(id: u64, value: &[u8]) -> Self {
        let mut value_buf = Buffer::from(value);
        let name = value_buf.get_string_utf8();
        let cardinality = value_buf.get_u8();

        PropertyKey {
            id,
            name,
            cardinality: Cardinality::from(cardinality),
        }
    }

    fn build_key(id: u64) -> Vec<u8> {
        let mut key_buf = Buffer::new();
        key_buf.put_u8(SchemaType::PropertyKey as u8);
        key_buf.put_u64(id);
        key_buf.to_vec()
    }

    fn get_prefix() -> Vec<u8> {
        let mut key_buf = Buffer::new();
        key_buf.put_u8(SchemaType::PropertyKey as u8);
        key_buf.to_vec()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serde_edge_label_model() {
        let pkm = PropertyKey {
            id: 1,
            name: "mock".to_string(),
            cardinality: Cardinality::Single,
        };
        let ser = pkm.serialize();
        let de = PropertyKey::deserialize(&ser.0, &ser.1);
        assert_eq!(pkm, de);
    }
}
