/// Implementation of the Glushkov's construction algorithm to build a linearized language out of a
/// regexp's HIR, and finaly convert this expression to a variable NFA.
use std::collections::LinkedList;
use std::rc::Rc;

use regex_syntax::hir;
use regex_syntax::hir::{GroupKind, HirKind, RepetitionKind};

use super::automata::{Atom, Label};
use super::mapping;

#[derive(Clone, Debug)]
pub struct GlushkovTerm {
    pub id: usize,
    pub label: Rc<Label>,
}

#[derive(Clone, Debug)]
pub struct GlushkovFactors {
    pub p: LinkedList<GlushkovTerm>,
    pub d: LinkedList<GlushkovTerm>,
    pub f: LinkedList<(GlushkovTerm, GlushkovTerm)>,
    pub g: bool,
}

#[derive(Clone, Debug)]
pub struct LocalLang {
    pub nb_terms: usize,
    pub factors: GlushkovFactors,
}

/// A local language is a regular language that can be identified with only its factors of size 2,
/// its prefixes and suffixes and wether it contains the empty word or not.
impl LocalLang {
    /// Return a language representing the input Hir.
    pub fn from_hir(hir: hir::Hir) -> LocalLang {
        match hir.into_kind() {
            HirKind::Empty => LocalLang::empty(),
            HirKind::Literal(lit) => LocalLang::label(Rc::new(Label::Atom(Atom::Literal(lit)))),
            HirKind::Class(class) => LocalLang::label(Rc::new(Label::Atom(Atom::Class(class)))),
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
                        let marker_open = Label::Assignation(mapping::Marker::Open(var.clone()));
                        let marker_close = Label::Assignation(mapping::Marker::Close(var));
                        LocalLang::concatenation(
                            LocalLang::label(Rc::new(marker_open)),
                            LocalLang::concatenation(lang, LocalLang::label(Rc::new(marker_close))),
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

    /// Register a new atom in the local language and return the associated state.
    fn register_label(&mut self, label: Rc<Label>) -> GlushkovTerm {
        self.nb_terms += 1;
        GlushkovTerm {
            id: self.nb_terms - 1,
            label,
        }
    }

    /// Return a local language representing an expression containing a single state.
    fn label(label: Rc<Label>) -> LocalLang {
        let mut lang = LocalLang::empty();
        let state = lang.register_label(label);

        lang.factors.p.push_back(state.clone());
        lang.factors.d.push_back(state);
        lang
    }

    /// Return an empty local language.
    fn empty() -> LocalLang {
        LocalLang {
            nb_terms: 0,
            factors: GlushkovFactors {
                p: LinkedList::new(),
                d: LinkedList::new(),
                f: LinkedList::new(),
                g: false,
            },
        }
    }

    /// Return a local language containing only the empty word.
    fn epsilon() -> LocalLang {
        LocalLang {
            nb_terms: 0,
            factors: GlushkovFactors {
                p: LinkedList::new(),
                d: LinkedList::new(),
                f: LinkedList::new(),
                g: true,
            },
        }
    }

    /// Return a local language containing the concatenation of words from the first and second
    /// input languages.
    fn concatenation(mut lang1: LocalLang, mut lang2: LocalLang) -> LocalLang {
        let nb_terms = lang1.nb_terms + lang2.nb_terms;
        let mut factors = GlushkovFactors {
            p: lang1.factors.p,
            d: lang2.factors.d,
            f: lang1.factors.f,
            g: lang1.factors.g && lang2.factors.g,
        };

        for x in &factors.d {
            for y in &lang2.factors.p {
                factors.f.push_back((x.clone(), y.clone()));
            }
        }

        if lang1.factors.g {
            factors.p.append(&mut lang2.factors.p);
        }

        if lang2.factors.g {
            factors.d.append(&mut lang1.factors.d);
        }

        factors.f.append(&mut lang2.factors.f);
        LocalLang { nb_terms, factors }
    }

    /// Return a local language containing words from the first or the second input languages.
    fn alternation(lang1: LocalLang, mut lang2: LocalLang) -> LocalLang {
        let nb_terms = lang1.nb_terms + lang2.nb_terms;
        let mut factors = lang1.factors;

        factors.p.append(&mut lang2.factors.p);
        factors.d.append(&mut lang2.factors.d);
        factors.f.append(&mut lang2.factors.f);
        factors.g = factors.g || lang2.factors.g;

        LocalLang { nb_terms, factors }
    }

    /// Return a local language containing the empty word and the input language.
    fn optional(mut lang: LocalLang) -> LocalLang {
        lang.factors.g = true;
        lang
    }

    /// Return a local language containing words made of one or more repetitions of words of the
    /// input language.
    fn closure(mut lang: LocalLang) -> LocalLang {
        for x in &lang.factors.d {
            for y in &lang.factors.p {
                lang.factors.f.push_back((x.clone(), y.clone()));
            }
        }

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
