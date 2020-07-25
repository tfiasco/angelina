use std::rc::Rc;

use crate::datamodel::property::Properties;
use crate::datamodel::property_key::PropertyKey;
use crate::datamodel::vertex::Vertex;
use crate::handlers::sled_engine::SledEngine;

static VERTEX_TABLE_NAME: &str = "VERTEX";

pub struct VertexHandler {
    pub(crate) engine: Rc<Box<SledEngine>>,
}

impl VertexHandler {
    pub fn create_vertex(&self, id: &str, label: u64) -> Vertex {
        let vertex = Vertex {
            id: id.to_string(),
            label,
            properties: Properties { data: Vec::new() },
        };
        let (key, value) = vertex.serialize();
        self.engine.insert(VERTEX_TABLE_NAME, &key, &value);
        vertex
    }

    pub fn remove_vertex(&self, id: &str) {
        let key = Vertex::build_key(id);
        self.engine.remove(VERTEX_TABLE_NAME, &key);
    }

    pub fn add_property(&self, vertex: &mut Vertex, property_key: &PropertyKey, value: &str) {
        let prop_id = self.generate_next_prop_id(&vertex.id);
        vertex
            .properties
            .add_property(property_key.id, prop_id, value);
        let (key, value) = vertex.serialize();
        self.engine.insert(VERTEX_TABLE_NAME, &key, &value);
    }

    pub fn remove_property(
        &self,
        vertex: &mut Vertex,
        property_key: &PropertyKey,
        prop_id: Vec<u64>,
    ) {
        vertex.properties.remove_property(property_key.id, prop_id);
        let (key, value) = vertex.serialize();
        self.engine.insert(VERTEX_TABLE_NAME, &key, &value);
    }

    pub fn get_vertex(&self, id: &str) -> Option<Vertex> {
        let key = Vertex::build_key(id);
        match self.engine.get(VERTEX_TABLE_NAME, &key) {
            Some(value) => Some(Vertex::deserialize_value(id, &value)),
            None => None,
        }
    }

    fn generate_next_prop_id(&self, vertex_id: &str) -> u64 {
        let auto_increment_key = format!("VERTEX_PROP_AUTO_INCREMENT_ID_{}", vertex_id);
        self.engine
            .increment(VERTEX_TABLE_NAME, &auto_increment_key)
    }
}

#[cfg(test)]
mod test {
    use crate::datamodel::base::Cardinality;

    use super::*;

    #[test]
    fn test_vertex_crud() {
        let engine = Rc::new(Box::new(SledEngine::new_tmp()));

        let handler = VertexHandler { engine };

        let key = PropertyKey {
            id: 1,
            name: "aa".to_string(),
            cardinality: Cardinality::Single,
        };

        let mut v = handler.create_vertex("xx_1", 1);
        handler.add_property(&mut v, &key, "test1");
        handler.add_property(&mut v, &key, "test2");
        let v2 = handler.get_vertex(&v.id).unwrap();
        assert_eq!(
            v2.properties
                .get_properties()
                .map(|x| { x.value })
                .collect::<Vec<String>>(),
            vec!["test1", "test2"]
        );
        assert_eq!(v2.id, v.id);

        handler.remove_property(&mut v, &key, vec![]);
        let v2 = handler.get_vertex(&v.id).unwrap();
        assert_eq!(
            v2.properties.get_properties().map(|x| { x.value }).count(),
            0
        );

        handler.remove_vertex(&v.id);
        assert_eq!(handler.get_vertex(&v.id), None);
    }
}
