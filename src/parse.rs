use std::rc::Rc;

use regex_syntax;
use regex_syntax::hir::GroupKind as LibGroup;
use regex_syntax::hir::HirKind as LibHir;
use regex_syntax::hir::RepetitionKind as LibRepKind;
use regex_syntax::hir::RepetitionRange as LibRepRange;

use super::automata::{Atom, Label};
use super::mapping::{Marker, Variable};

/// A simple Hir, with branchements of arity at most 2 and at least redondancy as possible.
#[derive(Clone, Debug)]
pub enum Hir {
    Empty,
    Label(Rc<Label>), // embeded into an Rc to avoid heavy copies of complex literals
    Concat(Box<Hir>, Box<Hir>),
    Alternation(Box<Hir>, Box<Hir>),
    Option(Box<Hir>),
    Closure(Box<Hir>),
}

impl Hir {
    pub fn from_regex(regex: &str) -> Hir {
        let lib_hir = regex_syntax::Parser::new().parse(regex).unwrap();
        Hir::from_lib_hir(lib_hir)
    }

    fn from_lib_hir(hir: regex_syntax::hir::Hir) -> Hir {
        match hir.into_kind() {
            LibHir::Empty => Hir::epsilon(),
            LibHir::Literal(lit) => Hir::label(Label::Atom(Atom::Literal(lit))),
            LibHir::Class(class) => Hir::label(Label::Atom(Atom::Class(class))),
            LibHir::Repetition(rep) => {
                let hir = Hir::from_lib_hir(*rep.hir);
                match rep.kind {
                    LibRepKind::ZeroOrOne => Hir::option(hir),
                    LibRepKind::ZeroOrMore => Hir::option(Hir::closure(hir)),
                    LibRepKind::OneOrMore => Hir::closure(hir),
                    LibRepKind::Range(range) => Hir::repetition(hir, range),
                }
            }
            LibHir::Group(group) => {
                let subtree = Hir::from_lib_hir(*group.hir);
                match group.kind {
                    LibGroup::NonCapturing | LibGroup::CaptureIndex(_) => subtree,
                    LibGroup::CaptureName { name, index: _ } => {
                        let var = Variable::new(name);
                        let marker_open = Label::Assignation(Marker::Open(var.clone()));
                        let marker_close = Label::Assignation(Marker::Close(var));

                        Hir::concat(
                            Hir::Concat(Box::new(Hir::label(marker_open)), Box::new(subtree)),
                            Hir::label(marker_close),
                        )
                    }
                }
            }
            LibHir::Concat(sub) => sub.into_iter().fold(Hir::epsilon(), |acc, branch| {
                Hir::concat(Hir::from_lib_hir(branch), acc)
            }),
            LibHir::Alternation(sub) => sub.into_iter().fold(Hir::Empty, |acc, branch| {
                Hir::alternation(Hir::from_lib_hir(branch), acc)
            }),
            other => panic!("Not implemented for: {:?}", other),
        }
    }

    fn epsilon() -> Hir {
        Hir::option(Hir::Empty)
    }

    fn label(label: Label) -> Hir {
        Hir::Label(Rc::new(label))
    }

    fn option(hir: Hir) -> Hir {
        Hir::Option(Box::new(hir))
    }

    fn concat(hir1: Hir, hir2: Hir) -> Hir {
        Hir::Concat(Box::new(hir1), Box::new(hir2))
    }

    fn alternation(hir1: Hir, hir2: Hir) -> Hir {
        Hir::Alternation(Box::new(hir1), Box::new(hir2))
    }

    fn closure(hir: Hir) -> Hir {
        Hir::Closure(Box::new(hir))
    }

    fn repetition(hir: Hir, range: LibRepRange) -> Hir {
        let (min, max) = match range {
            LibRepRange::Exactly(n) => (n, Some(n)),
            LibRepRange::AtLeast(n) => (n, None),
            LibRepRange::Bounded(m, n) => (m, Some(n)),
        };

        let mut result = Hir::epsilon();

        for i in 0..min {
            if i == min - 1 && max == None {
                result = Hir::concat(result, Hir::closure(hir.clone()));
            } else {
                result = Hir::concat(result, hir.clone());
            }
        }

        if let Some(max) = max {
            let mut optionals = Hir::epsilon();

            for _ in min..max {
                optionals = Hir::option(Hir::concat(hir.clone(), optionals));
            }

            result = Hir::concat(result, optionals);
        }

        result
    }
}
