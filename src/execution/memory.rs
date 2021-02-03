use std::collections::HashMap;

use crate::datamodel::edge::Edge;
use crate::datamodel::vertex::Vertex;

pub struct ExecutionMemory {
    vertices: HashMap<String, Vertex>,
    edges: HashMap<String, Edge>,
}

impl ExecutionMemory {
    pub fn new() -> Self {
        ExecutionMemory {
            vertices: Default::default(),
            edges: Default::default(),
        }
    }
}
