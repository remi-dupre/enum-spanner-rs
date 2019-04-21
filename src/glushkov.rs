/// Implementation of the Glushkov's construction algorithm to build a linearized language out of a
/// regexp's HIR, and finaly convert this expression to a variable NFA.
use super::automata::Atom;
use super::mapping;

use regex_syntax::hir;
use regex_syntax::hir::{GroupKind, HirKind, RepetitionKind};

#[derive(Debug)]
pub struct LocalLang {
    pub atoms: Vec<Atom>,
    pub factors: GlushkovFactors,
}

impl LocalLang {
    /// Return a language representing the input Hir.
    pub fn from_hir(hir: hir::Hir) -> LocalLang {
        match hir.kind() {
            HirKind::Empty => LocalLang::empty(),
            HirKind::Literal(lit) => LocalLang::atom(Atom::Literal(lit.clone())),
            HirKind::Class(class) => LocalLang::atom(Atom::Class(class.clone())),
            HirKind::Repetition(rep) => {
                let lang = LocalLang::from_hir(*rep.hir.clone());
                match rep.kind {
                    RepetitionKind::ZeroOrOne => LocalLang::optional(lang),
                    RepetitionKind::ZeroOrMore => LocalLang::optional(LocalLang::closure(lang)),
                    RepetitionKind::OneOrMore => LocalLang::closure(lang),
                    RepetitionKind::Range(ref range) => LocalLang::repetition(lang, range.clone()),
                }
            }
            HirKind::Group(group) => {
                let lang = LocalLang::from_hir(*group.hir.clone());
                match group.kind {
                    GroupKind::CaptureIndex(_) | GroupKind::NonCapturing => lang,
                    GroupKind::CaptureName { ref name, index: _ } => {
                        let var = mapping::Variable::new(name.clone());
                        LocalLang::concatenation(
                            LocalLang::atom(Atom::Marker(mapping::Marker::Open(var.clone()))),
                            LocalLang::concatenation(
                                lang,
                                LocalLang::atom(Atom::Marker(mapping::Marker::Close(var.clone()))),
                            ),
                        )
                    }
                }
            }
            HirKind::Concat(sub) => {
                let closure = |acc, x: &hir::Hir| {
                    LocalLang::concatenation(acc, LocalLang::from_hir(x.clone()))
                };
                sub.iter().fold(LocalLang::epsilon(), closure)
            }
            HirKind::Alternation(sub) => {
                let closure =
                    |acc, x: &hir::Hir| LocalLang::alternation(acc, LocalLang::from_hir(x.clone()));
                sub.iter().fold(LocalLang::empty(), closure)
            }
            other => panic!("Not implemented: {:?}", other),
        }
    }

    /// Given to local langages, unify their atom lists and return the two Glushkov factors
    /// with updated indexes to this new structure.
    fn unify_atoms(
        lang1: LocalLang,
        mut lang2: LocalLang,
    ) -> (Vec<Atom>, GlushkovFactors, GlushkovFactors) {
        let lang2_offset = lang1.atoms.len();
        lang2.factors.p = lang2.factors.p.iter().map(|x| x + lang2_offset).collect();
        lang2.factors.d = lang2.factors.d.iter().map(|x| x + lang2_offset).collect();
        lang2.factors.f = lang2
            .factors
            .f
            .iter()
            .map(|(x, y)| (x + lang2_offset, y + lang2_offset))
            .collect();

        let mut atoms = lang1.atoms;
        atoms.extend(lang2.atoms);

        (atoms, lang1.factors, lang2.factors)
    }

    /// Return a local language representing an expression containing a single
    /// atom.
    fn atom(atom: Atom) -> LocalLang {
        LocalLang {
            atoms: vec![atom],
            factors: GlushkovFactors::atom(),
        }
    }

    /// Return an empty local language.
    fn empty() -> LocalLang {
        LocalLang {
            atoms: Vec::new(),
            factors: GlushkovFactors::empty(),
        }
    }

    /// Return a local language containing only the empty word.
    fn epsilon() -> LocalLang {
        LocalLang {
            atoms: Vec::new(),
            factors: GlushkovFactors::epsilon(),
        }
    }

    /// Return a local language containing the concatenation of words from the
    /// first and second input languages.
    fn concatenation(lang1: LocalLang, lang2: LocalLang) -> LocalLang {
        let (atoms, factors1, factors2) = LocalLang::unify_atoms(lang1, lang2);
        LocalLang {
            atoms,
            factors: GlushkovFactors::concatenation(factors1, factors2),
        }
    }

    /// Return a local language containing words from the first or the second
    /// input languages.
    fn alternation(lang1: LocalLang, lang2: LocalLang) -> LocalLang {
        let (atoms, factors1, factors2) = LocalLang::unify_atoms(lang1, lang2);
        LocalLang {
            atoms,
            factors: GlushkovFactors::alternation(factors1, factors2),
        }
    }

    /// Return a local language containing the empty word and the input
    /// language.
    fn optional(mut lang: LocalLang) -> LocalLang {
        lang.factors = GlushkovFactors::optional(lang.factors);
        lang
    }

    /// Return a local language containing words made of one or more
    /// repetitions of words of the input language.
    fn closure(mut lang: LocalLang) -> LocalLang {
        lang.factors = GlushkovFactors::closure(lang.factors);
        lang
    }

    /// Return a local language containing words made of an interval-defined
    /// count of words from the input language.
    fn repetition(lang: LocalLang, range: hir::RepetitionRange) -> LocalLang {
        let (min, max) = match range {
            hir::RepetitionRange::Exactly(n) => (n, Some(n)),
            hir::RepetitionRange::AtLeast(n) => (n, None),
            hir::RepetitionRange::Bounded(m, n) => (m, Some(n)),
        };

        let factors = lang.factors;
        let mut result = GlushkovFactors::epsilon();

        for i in 0..min {
            if i == min - 1 && max == None {
                result = GlushkovFactors::concatenation(
                    result,
                    GlushkovFactors::closure(factors.clone()),
                );
            } else {
                result = GlushkovFactors::concatenation(result, factors.clone());
            }
        }

        if let Some(max) = max {
            let mut optionals = GlushkovFactors::empty();

            for _ in min..max {
                optionals = GlushkovFactors::optional(GlushkovFactors::concatenation(
                    factors.clone(),
                    optionals,
                ));
            }

            result = GlushkovFactors::concatenation(result, optionals);
        }

        LocalLang {
            atoms: lang.atoms,
            factors: result,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GlushkovFactors {
    pub p: Vec<usize>,
    pub d: Vec<usize>,
    pub f: Vec<(usize, usize)>,
    pub g: bool,
}

/// Please refer to the documentation of LocalLang for the aim of methods in this struct.
impl GlushkovFactors {
    fn atom() -> GlushkovFactors {
        GlushkovFactors {
            p: vec![0],
            d: vec![0],
            f: Vec::new(),
            g: false,
        }
    }

    fn empty() -> GlushkovFactors {
        GlushkovFactors {
            p: Vec::new(),
            d: Vec::new(),
            f: Vec::new(),
            g: false,
        }
    }

    fn epsilon() -> GlushkovFactors {
        GlushkovFactors {
            p: Vec::new(),
            d: Vec::new(),
            f: Vec::new(),
            g: true,
        }
    }

    fn concatenation(factors1: GlushkovFactors, factors2: GlushkovFactors) -> GlushkovFactors {
        let mut factors = factors1;

        for x in &factors.d {
            for y in &factors2.p {
                factors.f.push((*x, *y));
            }
        }

        factors.p.extend(factors2.p);
        factors.d.extend(factors2.d);
        factors.f.extend(factors2.f);
        factors.g = factors.g && factors2.g;

        factors
    }

    fn alternation(factors1: GlushkovFactors, factors2: GlushkovFactors) -> GlushkovFactors {
        let mut factors = factors1;
        factors.p.extend(factors2.p);
        factors.d.extend(factors2.d);
        factors.f.extend(factors2.f);
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
                factors.f.push((*x, *y));
            }
        }

        factors
    }
}
