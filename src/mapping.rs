use rand;

#[derive(Copy, Clone, Debug)]
pub struct Variable<'a> {
    id: u64,
    name: &'a str,
}

impl<'a> Variable<'a> {
    pub fn new(name: &'a str) -> Variable<'a> {
        Variable {
            id: rand::random(),
            name: name,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Marker<'a> {
    Close(Variable<'a>),
    Open(Variable<'a>),
}
