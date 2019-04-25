use regex_syntax::hir;
use std::fmt;

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
            Atom::Class(hir::Class::Unicode(class)) => class
                .iter()
                .any(|range| range.start() <= *a && *a <= range.end()),
            _ => panic!("Byte regex are not supported"),
        }
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Atom::Literal(hir::Literal::Unicode(x)) => write!(f, "{}", x),
            Atom::Class(hir::Class::Unicode(class)) => {
                write!(f, "[")?;
                for range in class.iter() {
                    write!(f, "{}-{}", range.start(), range.end())?;
                }
                write!(f, "]")
            }
            _ => panic!("Byte regex are not supported"),
        }
    }
}
