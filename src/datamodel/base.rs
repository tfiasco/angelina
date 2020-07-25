use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

pub trait BaseSchemaModel {
    fn serialize(&self) -> (Vec<u8>, Vec<u8>);
    fn deserialize(key: &[u8], value: &[u8]) -> Self;
    fn deserialize_value(id: u64, value: &[u8]) -> Self;
    fn build_key(id: u64) -> Vec<u8>;
    fn get_prefix() -> Vec<u8>;
}

#[repr(u8)]
pub enum SchemaType {
    VertexLabel = 0x01,
    EdgeLabel = 0x02,
    PropertyKey = 0x03,
}

#[repr(u8)]
pub enum ElementType {
    Vertex = 0x04,
    InEdge = 0x05,
    OutEdge = 0x06,
    MetaProperty = 0x07,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum EdgeMultiplicity {
    One2One = 0x01,
    One2Many,
    Many2One,
    Many2ManySimple,
    Many2ManyMulti,
}

impl Display for EdgeMultiplicity {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            EdgeMultiplicity::One2One => write!(f, "One2One"),
            EdgeMultiplicity::One2Many => write!(f, "One2Many"),
            EdgeMultiplicity::Many2One => write!(f, "Many2One"),
            EdgeMultiplicity::Many2ManySimple => write!(f, "Many2ManySimple"),
            EdgeMultiplicity::Many2ManyMulti => write!(f, "Many2ManyMulti"),
        }
    }
}

impl From<u8> for EdgeMultiplicity {
    fn from(i: u8) -> Self {
        match i {
            0x01 => EdgeMultiplicity::One2One,
            0x02 => EdgeMultiplicity::One2Many,
            0x03 => EdgeMultiplicity::Many2One,
            0x04 => EdgeMultiplicity::Many2ManySimple,
            0x05 => EdgeMultiplicity::Many2ManyMulti,
            _ => panic!("No Such EdgeMultiplicity"),
        }
    }
}

impl From<&str> for EdgeMultiplicity {
    fn from(value: &str) -> EdgeMultiplicity {
        match value.to_uppercase().as_str() {
            "ONE2ONE" => EdgeMultiplicity::One2One,
            "ONE2MANY" => EdgeMultiplicity::One2Many,
            "MANY2ONE" => EdgeMultiplicity::Many2One,
            "MANY2MANYSIMPLE" => EdgeMultiplicity::Many2ManySimple,
            "MANY2MANYMULTI" => EdgeMultiplicity::Many2ManyMulti,
            _ => panic!("No Such EdgeMultiplicity"),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Cardinality {
    Single = 0x01,
    List,
    Set,
}

impl Display for Cardinality {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Cardinality::Single => write!(f, "Single"),
            Cardinality::List => write!(f, "List"),
            Cardinality::Set => write!(f, "Set"),
        }
    }
}

impl From<u8> for Cardinality {
    fn from(value: u8) -> Cardinality {
        match value {
            0x01 => Cardinality::Single,
            0x02 => Cardinality::List,
            0x03 => Cardinality::Set,
            _ => panic!("No Such Cardinality"),
        }
    }
}

impl From<&str> for Cardinality {
    fn from(value: &str) -> Cardinality {
        match value.to_uppercase().as_str() {
            "SINGLE" => Cardinality::Single,
            "LIST" => Cardinality::List,
            "SET" => Cardinality::Set,
            _ => panic!("No Such Cardinality"),
        }
    }
}

pub enum EdgeDirection {
    Out,
    In,
}
