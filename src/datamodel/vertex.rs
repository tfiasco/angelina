use crate::datamodel::base::ElementType;
use crate::datamodel::buffer::Buffer;
use crate::datamodel::property::Properties;

#[derive(Debug, Eq, PartialEq)]
pub struct Vertex {
    pub id: String,
    pub label: u64,
    pub properties: Properties,
}


impl Vertex {
    pub fn serialize(&self) -> (Vec<u8>, Vec<u8>) {
        let key = Self::build_key(&self.id);
        let mut value_buf = Buffer::new();
        value_buf.put_u64(self.label);
        value_buf.put_slice(&self.properties.data);
        (key, value_buf.to_vec())
    }

    pub fn deserialize(key: &[u8], value: &[u8]) -> Self {
        let mut key_buf = Buffer::from(key);
        key_buf.get_u8();      // SchemaType
        let id = key_buf.get_string_utf8();

        Self::deserialize_value(&id, value)
    }

    pub fn deserialize_value(id: &str, value: &[u8]) -> Self {
        let mut value_buf = Buffer::from(value);
        let label = value_buf.get_u64();
        let properties = value_buf.to_vec();

        Vertex {
            id: id.to_string(),
            label,
            properties: Properties { data: properties },
        }
    }

    pub fn build_key(id: &str) -> Vec<u8> {
        let mut key_buf = Buffer::new();
        key_buf.put_u8(ElementType::Vertex as u8);
        key_buf.put_string(&id);
        key_buf.to_vec()
    }
}
