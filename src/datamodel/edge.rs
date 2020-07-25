use crate::datamodel::base::{EdgeDirection, ElementType};
use crate::datamodel::buffer::Buffer;
use crate::datamodel::property::Properties;

#[derive(Debug, Eq, PartialEq)]
pub struct Edge {
    pub src_vertex_id: String,
    pub dst_vertex_id: String,
    pub edge_id: u64,
    pub label: u64,
    pub properties: Properties,
}


impl Edge {
    pub fn serialize(&self, direction: EdgeDirection) -> (Vec<u8>, Vec<u8>) {
        let key = self.generate_key(direction);
        let mut value_buf = Buffer::from(&self.properties.data);

        (key, value_buf.to_vec())
    }

    pub fn deserialize(key: &[u8], value: &[u8]) -> Edge {
        let mut key_buf = Buffer::from(key);
        let element_type = key_buf.get_u8();      // ElementType
        let first_id = key_buf.get_string_utf8();
        let edge_label = key_buf.get_u64();
        let second_id = key_buf.get_string_utf8();
        let edge_id = key_buf.get_u64();

        match element_type {
            _ if element_type == ElementType::OutEdge as u8 => {
                Edge {
                    src_vertex_id: first_id,
                    dst_vertex_id: second_id,
                    edge_id,
                    label: edge_label,
                    properties: Properties { data: value.to_owned() },
                }
            }
            _ if element_type == ElementType::InEdge as u8 => {
                Edge {
                    src_vertex_id: second_id,
                    dst_vertex_id: first_id,
                    edge_id,
                    label: edge_label,
                    properties: Properties { data: value.to_owned() },
                }
            }
            _ => panic!("ElementType Error!")
        }
    }

    pub fn generate_key(&self, direction: EdgeDirection) -> Vec<u8> {
        Self::build_key(&self.src_vertex_id, &self.dst_vertex_id, self.label,
                        self.edge_id, direction)
    }

    pub fn build_key(src_id: &str, dst_id: &str, label: u64,
                     edge_id: u64, direction: EdgeDirection) -> Vec<u8> {
        let (element_type, first_id, second_id) = match direction {
            EdgeDirection::Out => (ElementType::OutEdge, src_id, dst_id),
            EdgeDirection::In => (ElementType::InEdge, dst_id, src_id)
        };

        let mut key_buf = Buffer::new();
        key_buf.put_u8(element_type as u8);
        key_buf.put_string(first_id);
        key_buf.put_u64(label);
        key_buf.put_string(second_id);
        key_buf.put_u64(edge_id);
        key_buf.to_vec()
    }
}
