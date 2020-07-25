use crate::datamodel::base::{BaseSchemaModel, SchemaType};
use crate::datamodel::buffer::Buffer;

#[derive(Debug, Eq, PartialEq)]
pub struct VertexLabel {
    pub(crate) id: u64,
    pub(crate) name: String,
}

impl BaseSchemaModel for VertexLabel {
    fn serialize(&self) -> (Vec<u8>, Vec<u8>) {
        let key = Self::build_key(self.id);

        let mut value_buf = Buffer::new();
        value_buf.put_string(&self.name);
        (key, value_buf.to_vec())
    }

    fn deserialize(key: &[u8], value: &[u8]) -> Self {
        let mut key_buf = Buffer::from(key);
        key_buf.get_u8(); // SchemaType
        let id = key_buf.get_u64();

        Self::deserialize_value(id, value)
    }

    fn deserialize_value(id: u64, value: &[u8]) -> Self {
        let name = Buffer::from(value).get_string_utf8();

        VertexLabel { id, name }
    }

    fn build_key(id: u64) -> Vec<u8> {
        let mut key_buf = Buffer::new();
        key_buf.put_u8(SchemaType::VertexLabel as u8);
        key_buf.put_u64(id);
        key_buf.to_vec()
    }

    fn get_prefix() -> Vec<u8> {
        let mut key_buf = Buffer::new();
        key_buf.put_u8(SchemaType::VertexLabel as u8);
        key_buf.to_vec()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serde_vertex_model() {
        let vlm = VertexLabel {
            id: 1,
            name: "mock".to_string(),
        };
        let ser = vlm.serialize();
        let de = VertexLabel::deserialize(&ser.0, &ser.1);
        assert_eq!(vlm, de);
    }
}
