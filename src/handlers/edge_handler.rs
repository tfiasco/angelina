use std::rc::Rc;

use crate::datamodel::base::EdgeDirection;
use crate::datamodel::edge::Edge;
use crate::datamodel::property::Properties;
use crate::datamodel::property_key::PropertyKey;
use crate::handlers::sled_engine::SledEngine;

static EDGE_TABLE_NAME: &str = "EDGE";

pub struct EdgeHandler {
    pub(crate) engine: Rc<Box<SledEngine>>,
}

impl EdgeHandler {
    pub fn create_edge(&self, src_vertex_id: &str, dst_vertex_id: &str, label: u64) -> Edge {
        let edge_id = self.generate_next_edge_id();
        let edge = Edge {
            src_vertex_id: src_vertex_id.to_owned(),
            dst_vertex_id: dst_vertex_id.to_owned(),
            edge_id,
            label,
            properties: Properties { data: Vec::new() },
        };
        let (out_key, out_value) = edge.serialize(EdgeDirection::Out);
        let (in_key, in_value) = edge.serialize(EdgeDirection::In);
        self.engine.insert(EDGE_TABLE_NAME, &in_key, &in_value);
        self.engine.insert(EDGE_TABLE_NAME, &out_key, &out_value);
        edge
    }

    pub fn remove_edge(&self, edge: &Edge) {
        let in_key = edge.generate_key(EdgeDirection::In);
        let out_key = edge.generate_key(EdgeDirection::Out);
        self.engine.remove(EDGE_TABLE_NAME, &in_key);
        self.engine.remove(EDGE_TABLE_NAME, &out_key);
    }

    pub fn add_property(&self, edge: &mut Edge, property_key: &PropertyKey, value: &str) {
        let prop_id = self.generate_next_prop_id(edge.edge_id);
        edge.properties
            .add_property(property_key.id, prop_id, value);
        let (out_key, out_value) = edge.serialize(EdgeDirection::Out);
        let (in_key, in_value) = edge.serialize(EdgeDirection::In);
        self.engine.insert(EDGE_TABLE_NAME, &in_key, &in_value);
        self.engine.insert(EDGE_TABLE_NAME, &out_key, &out_value);
    }

    pub fn remove_property(&self, edge: &mut Edge, property_key: &PropertyKey, prop_id: Vec<u64>) {
        edge.properties.remove_property(property_key.id, prop_id);
        let (out_key, out_value) = edge.serialize(EdgeDirection::Out);
        let (in_key, in_value) = edge.serialize(EdgeDirection::In);
        self.engine.insert(EDGE_TABLE_NAME, &in_key, &in_value);
        self.engine.insert(EDGE_TABLE_NAME, &out_key, &out_value);
    }

    pub fn get_edge(
        &self,
        src_id: &str,
        dst_id: &str,
        label: u64,
        edge_id: u64,
        direction: EdgeDirection,
    ) -> Option<Edge> {
        let key = Edge::build_key(src_id, dst_id, label, edge_id, direction);
        match self.engine.get(EDGE_TABLE_NAME, &key) {
            Some(value) => Some(Edge::deserialize(&key, &value)),
            None => None,
        }
    }

    fn generate_next_edge_id(&self) -> u64 {
        let auto_increment_key = "EDGE_AUTO_INCREMENT_ID";
        self.engine.increment(EDGE_TABLE_NAME, &auto_increment_key)
    }

    fn generate_next_prop_id(&self, edge_id: u64) -> u64 {
        let auto_increment_key = format!("EDGE_PROP_AUTO_INCREMENT_ID_{}", edge_id);
        self.engine.increment(EDGE_TABLE_NAME, &auto_increment_key)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::datamodel::base::Cardinality;

    #[test]
    fn test_edge_crud() {
        let engine = Rc::new(Box::new(SledEngine::new_tmp()));

        let handler = EdgeHandler { engine };

        let key = PropertyKey {
            id: 1,
            name: "aa".to_string(),
            cardinality: Cardinality::Single,
        };

        let mut e = handler.create_edge("xx_1", "xx_2", 1);
        handler.add_property(&mut e, &key, "test1");
        handler.add_property(&mut e, &key, "test2");
        let e2 = handler
            .get_edge(
                &e.src_vertex_id,
                &e.dst_vertex_id,
                e.label,
                e.edge_id,
                EdgeDirection::Out,
            )
            .unwrap();
        assert_eq!(
            e2.properties
                .get_properties()
                .map(|x| { x.value })
                .collect::<Vec<String>>(),
            vec!["test1", "test2"]
        );
        assert_eq!(e2.edge_id, e.edge_id);

        handler.remove_property(&mut e, &key, vec![]);
        let e2 = handler
            .get_edge(
                &e.src_vertex_id,
                &e.dst_vertex_id,
                e.label,
                e.edge_id,
                EdgeDirection::Out,
            )
            .unwrap();
        assert_eq!(
            e2.properties.get_properties().map(|x| { x.value }).count(),
            0
        );

        handler.remove_edge(&e);
        assert_eq!(
            handler.get_edge(
                &e.src_vertex_id,
                &e.dst_vertex_id,
                e.label,
                e.edge_id,
                EdgeDirection::Out
            ),
            None
        );
    }
}
