pub struct QueryOutput {
    pub(crate) columns: Vec<String>,
    pub items: Box<dyn Iterator<Item = Vec<String>>>,
}
