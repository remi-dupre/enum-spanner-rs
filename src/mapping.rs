use rand;

#[derive(Clone, Debug)]
pub struct Variable {
    id: u64,
    name: String,
}

impl Variable {
    pub fn new(name: String) -> Variable {
        Variable {
            id: rand::random(),
            name: name,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Marker {
    Close(Variable),
    Open(Variable),
}
