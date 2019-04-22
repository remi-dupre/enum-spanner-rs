/// Implementation of the Glushkov's construction algorithm to build a linearized language out of a
/// regexp's HIR, and finaly convert this expression to a variable NFA.
use std::collections::LinkedList;
use std::rc::Rc;

use regex_syntax::hir;
use regex_syntax::hir::{GroupKind, HirKind, RepetitionKind};

use super::automata::{Atom, Label, LabelKind};
use super::mapping;

#[derive(Clone, Debug)]
pub struct LocalLang {
    pub nb_labels: usize,
    pub factors: GlushkovFactors,
}

impl LocalLang {
    /// Return a language representing the input Hir.
    pub fn from_hir(hir: hir::Hir) -> LocalLang {
        match hir.into_kind() {
            HirKind::Empty => LocalLang::empty(),
            HirKind::Literal(lit) => LocalLang::label(LabelKind::Atom(Atom::Literal(lit))),
            HirKind::Class(class) => LocalLang::label(LabelKind::Atom(Atom::Class(class))),
            HirKind::Repetition(rep) => {
                let lang = LocalLang::from_hir(*rep.hir);
                match rep.kind {
                    RepetitionKind::ZeroOrOne => LocalLang::optional(lang),
                    RepetitionKind::ZeroOrMore => LocalLang::optional(LocalLang::closure(lang)),
                    RepetitionKind::OneOrMore => LocalLang::closure(lang),
                    RepetitionKind::Range(range) => LocalLang::repetition(lang, range),
                }
            }
            HirKind::Group(group) => {
                let lang = LocalLang::from_hir(*group.hir);
                match group.kind {
                    GroupKind::CaptureIndex(_) | GroupKind::NonCapturing => lang,
                    GroupKind::CaptureName { name, index: _ } => {
                        let var = mapping::Variable::new(name);
                        let marker_open = mapping::Marker::Open(var.clone());
                        let marker_close = mapping::Marker::Close(var);
                        LocalLang::concatenation(
                            LocalLang::label(LabelKind::Assignation(marker_open)),
                            LocalLang::concatenation(
                                lang,
                                LocalLang::label(LabelKind::Assignation(marker_close)),
                            ),
                        )
                    }
                }
            }
            HirKind::Concat(sub) => {
                let closure = |acc, x| LocalLang::concatenation(acc, LocalLang::from_hir(x));
                sub.into_iter().fold(LocalLang::epsilon(), closure)
            }
            HirKind::Alternation(sub) => {
                let closure = |acc, x| LocalLang::alternation(acc, LocalLang::from_hir(x));
                sub.into_iter().fold(LocalLang::empty(), closure)
            }
            other => panic!("Not implemented: {:?}", other),
        }
    }

    /// Register a new atom in the local language for later use

    fn register_label(&mut self, kind: LabelKind) -> Label {
        self.nb_labels += 1;
        Label {
            id: self.nb_labels - 1,
            kind,
        }
    }

    /// Return a local language representing an expression containing a single atom.
    fn label(kind: LabelKind) -> LocalLang {
        let mut lang = LocalLang::empty();
        let label = Rc::new(lang.register_label(kind));

        lang.factors.p.push_back(label.clone());
        lang.factors.d.push_back(label);
        lang
    }

    /// Return an empty local language.
    fn empty() -> LocalLang {
        LocalLang {
            nb_labels: 0,
            factors: GlushkovFactors::empty(),
        }
    }

    /// Return a local language containing only the empty word.
    fn epsilon() -> LocalLang {
        LocalLang {
            nb_labels: 0,
            factors: GlushkovFactors::epsilon(),
        }
    }

    /// Return a local language containing the concatenation of words from the first and second
    /// input languages.
    fn concatenation(lang1: LocalLang, lang2: LocalLang) -> LocalLang {
        LocalLang {
            nb_labels: lang1.nb_labels + lang2.nb_labels,
            factors: GlushkovFactors::concatenation(lang1.factors, lang2.factors),
        }
    }

    /// Return a local language containing words from the first or the second input languages.
    fn alternation(lang1: LocalLang, lang2: LocalLang) -> LocalLang {
        LocalLang {
            nb_labels: lang1.nb_labels + lang2.nb_labels,
            factors: GlushkovFactors::alternation(lang1.factors, lang2.factors),
        }
    }

    /// Return a local language containing the empty word and the input language.
    fn optional(mut lang: LocalLang) -> LocalLang {
        lang.factors = GlushkovFactors::optional(lang.factors);
        lang
    }

    /// Return a local language containing words made of one or more repetitions of words of the
    /// input language.
    fn closure(mut lang: LocalLang) -> LocalLang {
        lang.factors = GlushkovFactors::closure(lang.factors);
        lang
    }

    /// Return a local language containing words made of an interval-defined count of words from
    /// the input language.
    fn repetition(lang: LocalLang, range: hir::RepetitionRange) -> LocalLang {
        let (min, max) = match range {
            hir::RepetitionRange::Exactly(n) => (n, Some(n)),
            hir::RepetitionRange::AtLeast(n) => (n, None),
            hir::RepetitionRange::Bounded(m, n) => (m, Some(n)),
        };

        let mut result = LocalLang::epsilon();

        for i in 0..min {
            if i == min - 1 && max == None {
                result = LocalLang::concatenation(result, LocalLang::closure(lang.clone()));
            } else {
                result = LocalLang::concatenation(result, lang.clone());
            }
        }

        if let Some(max) = max {
            let mut optionals = LocalLang::empty();

            for _ in min..max {
                optionals = LocalLang::optional(LocalLang::concatenation(lang.clone(), optionals));
            }

            result = LocalLang::concatenation(result, optionals);
        }

        result
    }
}

#[derive(Clone, Debug)]
pub struct GlushkovFactors {
    pub p: LinkedList<Rc<Label>>,
    pub d: LinkedList<Rc<Label>>,
    pub f: LinkedList<(Rc<Label>, Rc<Label>)>,
    pub g: bool,
}

/// Please refer to the documentation of LocalLang for the aim of methods in this struct.
impl GlushkovFactors {
    fn atom(atom: &Rc<Label>) -> GlushkovFactors {
        let mut list = LinkedList::new();
        list.push_back(atom.clone());

        GlushkovFactors {
            p: list.clone(),
            d: list,
            f: LinkedList::new(),
            g: false,
        }
    }

    fn empty() -> GlushkovFactors {
        GlushkovFactors {
            p: LinkedList::new(),
            d: LinkedList::new(),
            f: LinkedList::new(),
            g: false,
        }
    }

    fn epsilon() -> GlushkovFactors {
        GlushkovFactors {
            p: LinkedList::new(),
            d: LinkedList::new(),
            f: LinkedList::new(),
            g: true,
        }
    }

    fn concatenation(
        mut factors1: GlushkovFactors,
        mut factors2: GlushkovFactors,
    ) -> GlushkovFactors {
        let mut factors = GlushkovFactors {
            p: factors1.p,
            d: factors2.d,
            f: factors1.f,
            g: factors1.g && factors2.g,
        };

        for x in &factors.d {
            for y in &factors2.p {
                factors.f.push_back((x.clone(), y.clone()));
            }
        }

        if factors1.g {
            factors.p.append(&mut factors2.p);
        }

        if factors2.g {
            factors.d.append(&mut factors1.d);
        }

        factors.f.append(&mut factors2.f);
        factors
    }

    fn alternation(factors1: GlushkovFactors, mut factors2: GlushkovFactors) -> GlushkovFactors {
        let mut factors = factors1;
        factors.p.append(&mut factors2.p);
        factors.d.append(&mut factors2.d);
        factors.f.append(&mut factors2.f);
        factors.g = factors.g || factors2.g;
        factors
    }

    fn optional(mut factors: GlushkovFactors) -> GlushkovFactors {
        factors.g = true;
        factors
    }

    fn closure(mut factors: GlushkovFactors) -> GlushkovFactors {
        for x in &factors.d {
            for y in &factors.p {
                factors.f.push_back((x.clone(), y.clone()));
            }
        }

        factors
    }
}
