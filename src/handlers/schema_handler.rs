extern crate bytes;

use std::rc::Rc;

use crate::datamodel::base::{BaseSchemaModel, Cardinality, EdgeMultiplicity};
use crate::datamodel::edge_label::EdgeLabel;
use crate::datamodel::property_key::PropertyKey;
use crate::datamodel::vertex_label::VertexLabel;
use crate::handlers::sled_engine::SledEngine;

static SCHEMA_TABLE_NAME: &str = "SCHEMA";
static AUTO_INCREMENT_SCHEMA_ID_KEY: &str = "SCHEMA_ID";

pub struct SchemaHandler {
    pub(crate) engine: Rc<Box<SledEngine>>,
}

impl SchemaHandler {
    // ============== VERTEX LABEL ==============
    pub fn create_vertex_label(&self, name: &str) -> u64 {
        let id = self.generate_next_id();
        let model = VertexLabel {
            id,
            name: name.to_owned(),
        };

        let (key, value) = model.serialize();
        self.engine.insert(SCHEMA_TABLE_NAME, &key, &value);
        id
    }

    pub fn get_vertex_label(&self, id: u64) -> Option<VertexLabel> {
        match self
            .engine
            .get(SCHEMA_TABLE_NAME, &VertexLabel::build_key(id))
        {
            Some(value) => Some(VertexLabel::deserialize_value(id, &value)),
            None => None,
        }
    }

    pub fn get_vertex_labels(&self) -> Vec<VertexLabel> {
        self.engine
            .open_tree(SCHEMA_TABLE_NAME)
            .scan_prefix(VertexLabel::get_prefix())
            .map(|res| {
                let key_value = res.unwrap();
                VertexLabel::deserialize(&key_value.0, &key_value.1)
            })
            .collect()
    }

    pub fn update_vertex_label(&self, id: u64, name: &str) {
        let model = VertexLabel {
            id,
            name: name.to_owned(),
        };
        let (key, value) = model.serialize();
        self.engine.insert(SCHEMA_TABLE_NAME, &key, &value);
    }

    pub fn remove_vertex_label(&self, id: u64) {
        let stored_id = VertexLabel::build_key(id);
        self.engine.remove(SCHEMA_TABLE_NAME, &stored_id);
    }

    pub fn get_vertex_label_by_name(&self, name: &str) -> Option<VertexLabel> {
        self.get_vertex_labels()
            .into_iter()
            .filter(|label| label.name == name)
            .next()
    }

    // ============== EDGE LABEL ==============
    pub fn create_edge_label(&self, name: &str, multiplicity: EdgeMultiplicity) -> u64 {
        let id = self.generate_next_id();
        let model = EdgeLabel {
            id,
            name: name.to_owned(),
            multiplicity,
        };

        let (key, value) = model.serialize();
        self.engine.insert(SCHEMA_TABLE_NAME, &key, &value);
        id
    }

    pub fn get_edge_label(&self, id: u64) -> Option<EdgeLabel> {
        match self
            .engine
            .get(SCHEMA_TABLE_NAME, &EdgeLabel::build_key(id))
        {
            Some(value) => Some(EdgeLabel::deserialize_value(id, &value)),
            None => None,
        }
    }

    pub fn get_edge_labels(&self) -> Vec<EdgeLabel> {
        self.engine
            .open_tree(SCHEMA_TABLE_NAME)
            .scan_prefix(EdgeLabel::get_prefix())
            .map(|res| {
                let key_value = res.unwrap();
                EdgeLabel::deserialize(&key_value.0, &key_value.1)
            })
            .collect()
    }

    pub fn update_edge_label(&self, id: u64, name: &str) {
        let stored_key = EdgeLabel::build_key(id);
        self.engine.open_tree(SCHEMA_TABLE_NAME).update_and_fetch(
            stored_key,
            |old_value| -> Option<Vec<u8>> {
                match old_value {
                    Some(value) => {
                        let old_edge_label = EdgeLabel::deserialize_value(id, value);
                        let new_edge_label = EdgeLabel {
                            id,
                            name: name.to_owned(),
                            multiplicity: old_edge_label.multiplicity,
                        };
                        Some(new_edge_label.serialize().1)
                    }
                    None => panic!("No such EdgeLabel"),
                }
            },
        );
    }

    pub fn remove_edge_label(&self, id: u64) {
        let stored_id = EdgeLabel::build_key(id);
        self.engine.remove(SCHEMA_TABLE_NAME, &stored_id);
    }

    pub fn get_edge_label_by_name(&self, name: &str) -> Option<EdgeLabel> {
        self.get_edge_labels()
            .into_iter()
            .filter(|label| label.name == name)
            .next()
    }

    // ============== PROPERTY KEY ==============
    pub fn create_property_key(&self, name: &str, cardinality: Cardinality) -> u64 {
        let id = self.generate_next_id();
        let model = PropertyKey {
            id,
            name: name.to_owned(),
            cardinality,
        };

        let (key, value) = model.serialize();
        self.engine.insert(SCHEMA_TABLE_NAME, &key, &value);
        id
    }

    pub fn get_property_key(&self, id: u64) -> Option<PropertyKey> {
        match self
            .engine
            .get(SCHEMA_TABLE_NAME, &PropertyKey::build_key(id))
        {
            Some(value) => Some(PropertyKey::deserialize_value(id, &value)),
            None => None,
        }
    }

    pub fn get_property_keys(&self) -> Vec<PropertyKey> {
        self.engine
            .open_tree(SCHEMA_TABLE_NAME)
            .scan_prefix(PropertyKey::get_prefix())
            .map(|res| {
                let key_value = res.unwrap();
                PropertyKey::deserialize(&key_value.0, &key_value.1)
            })
            .collect()
    }

    pub fn update_property_key(&self, id: u64, name: &str) {
        let stored_key = PropertyKey::build_key(id);
        self.engine.open_tree(SCHEMA_TABLE_NAME).update_and_fetch(
            stored_key,
            |old_value| -> Option<Vec<u8>> {
                match old_value {
                    Some(value) => {
                        let old_property_key = PropertyKey::deserialize_value(id, value);
                        let new_property_key = PropertyKey {
                            id,
                            name: name.to_owned(),
                            cardinality: old_property_key.cardinality,
                        };
                        Some(new_property_key.serialize().1)
                    }
                    None => panic!("No such Property Key"),
                }
            },
        );
    }

    pub fn remove_property_key(&self, id: u64) {
        let stored_id = PropertyKey::build_key(id);
        self.engine.remove(SCHEMA_TABLE_NAME, &stored_id);
    }

    pub fn get_property_key_by_name(&self, name: &str) -> Option<PropertyKey> {
        self.get_property_keys()
            .into_iter()
            .filter(|key| key.name == name)
            .next()
    }

    fn generate_next_id(&self) -> u64 {
        self.engine
            .increment(SCHEMA_TABLE_NAME, AUTO_INCREMENT_SCHEMA_ID_KEY)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vertex_label_crud() {
        let engine = Rc::new(Box::new(SledEngine::new_tmp()));
        let name = "name";
        let name2 = "name2";
        let handler = SchemaHandler { engine };
        let id = handler.create_vertex_label(name);
        let vertex_label = handler.get_vertex_label(id).unwrap();
        assert_eq!(
            vertex_label,
            VertexLabel {
                id,
                name: name.to_owned(),
            }
        );

        handler.update_vertex_label(id, name2);
        let vertex_label = handler.get_vertex_label(id).unwrap();
        assert_eq!(
            vertex_label,
            VertexLabel {
                id,
                name: name2.to_owned(),
            }
        );

        handler.remove_vertex_label(id);
        let vertex_label = handler.get_vertex_label(id);
        assert_eq!(vertex_label, None);
    }

    #[test]
    fn test_edge_label_crud() {
        let engine = Rc::new(Box::new(SledEngine::new_tmp()));
        let name = "name";
        let name2 = "name2";
        let multiplicity = EdgeMultiplicity::One2One;
        let handler = SchemaHandler { engine };
        let id = handler.create_edge_label(name, multiplicity);
        let label = handler.get_edge_label(id).unwrap();
        assert_eq!(
            label,
            EdgeLabel {
                id,
                name: name.to_owned(),
                multiplicity,
            }
        );

        handler.update_edge_label(id, name2);
        let label = handler.get_edge_label(id).unwrap();
        assert_eq!(
            label,
            EdgeLabel {
                id,
                name: name2.to_owned(),
                multiplicity,
            }
        );

        handler.remove_edge_label(id);
        let label = handler.get_edge_label(id);
        assert_eq!(label, None);
    }

    #[test]
    fn test_property_key_crud() {
        let engine = Rc::new(Box::new(SledEngine::new_tmp()));
        let name = "name";
        let name2 = "name2";
        let cardinality = Cardinality::Single;
        let handler = SchemaHandler { engine };
        let id = handler.create_property_key(name, cardinality);
        let p = handler.get_property_key(id).unwrap();
        assert_eq!(
            p,
            PropertyKey {
                id,
                name: name.to_owned(),
                cardinality,
            }
        );

        handler.update_property_key(id, name2);
        let p = handler.get_property_key(id).unwrap();
        assert_eq!(
            p,
            PropertyKey {
                id,
                name: name2.to_owned(),
                cardinality,
            }
        );

        handler.remove_property_key(id);
        let p = handler.get_property_key(id);
        assert_eq!(p, None);
    }
}
