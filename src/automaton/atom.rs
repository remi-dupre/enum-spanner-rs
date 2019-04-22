use regex_syntax::hir;

/// Represent a set of characters as an union of ranges.
#[derive(Debug)]
pub enum Atom {
    Literal(hir::Literal),
    Class(hir::Class),
}

impl Atom {
    /// Check if a unicode character matches an atom.
    pub fn is_match(&self, a: &char) -> bool {
        match self {
            Atom::Literal(hir::Literal::Unicode(x)) => a == x,
            Atom::Class(hir::Class::Unicode(class)) => class.iter().fold(false, |acc, range| {
                acc || (range.start() <= *a && *a <= range.end())
            }),
            _ => panic!("This regex was compiled for unicode datas"),
        }
    }
}
