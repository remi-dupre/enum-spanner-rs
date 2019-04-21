/// Implementation of the Glushkov's construction algorithm to build a
/// linearized language out of a regexp's HIR, and finaly convert this
/// expression to a variable NFA.
use regex_syntax::hir;

#[derive(Debug)]
pub enum Atom<'a> {
    Literal(&'a hir::Literal),
    Class(&'a hir::Class),
}

impl<'a> Copy for Atom<'a> {}

impl<'a> Clone for Atom<'a> {
    fn clone(&self) -> Atom<'a> {
        *self
    }
}

#[derive(Debug)]
pub struct LocalLang<'a> {
    atoms: Vec<Atom<'a>>,
    p: Vec<usize>,
    d: Vec<usize>,
    f: Vec<(usize, usize)>,
    g: bool,
}

impl<'a> LocalLang<'a> {
    /// Return a language representing the input Hir.
    pub fn from_hir(hir: &hir::Hir) -> LocalLang {
        match hir.kind() {
            hir::HirKind::Empty => LocalLang::empty(),
            hir::HirKind::Literal(lit) => LocalLang::atom(Atom::Literal(lit)),
            hir::HirKind::Class(class) => LocalLang::atom(Atom::Class(class)),
            hir::HirKind::Repetition(rep) => {
                let lang = LocalLang::from_hir(&(*rep.hir));
                match rep.kind {
                    hir::RepetitionKind::ZeroOrOne => LocalLang::optional(&lang),
                    hir::RepetitionKind::ZeroOrMore => {
                        LocalLang::optional(&LocalLang::closure(&lang))
                    }
                    hir::RepetitionKind::OneOrMore => LocalLang::closure(&lang),
                    hir::RepetitionKind::Range(ref range) => LocalLang::repetition(lang, &range),
                }
            }
            hir::HirKind::Concat(sub) => {
                let closure = |acc, x| LocalLang::concatenation(&acc, &LocalLang::from_hir(x));

                sub.iter().fold(LocalLang::epsilon(), closure)
            }
            hir::HirKind::Alternation(sub) => {
                let closure = |acc, x| LocalLang::alternation(&acc, &LocalLang::from_hir(x));

                sub.iter().fold(LocalLang::empty(), closure)
            }
            other => panic!("Not implemented: {:?}", other),
        }
    }

    /// Return a local language representing an expression containing a single
    /// atom.
    fn atom(atom: Atom<'a>) -> LocalLang<'a> {
        LocalLang {
            atoms: vec![atom],
            p: vec![0],
            d: vec![0],
            f: Vec::new(),
            g: false,
        }
    }

    /// Return an empty local language.
    fn empty() -> LocalLang<'a> {
        LocalLang {
            atoms: Vec::new(),
            p: Vec::new(),
            d: Vec::new(),
            f: Vec::new(),
            g: false,
        }
    }

    /// Return a local language containing only the empty word.
    fn epsilon() -> LocalLang<'a> {
        LocalLang {
            atoms: Vec::new(),
            p: Vec::new(),
            d: Vec::new(),
            f: Vec::new(),
            g: true,
        }
    }

    /// Return a local language containing the concatenation of words from the
    /// first and second input languages.
    fn concatenation(lang1: &LocalLang<'a>, lang2: &LocalLang<'a>) -> LocalLang<'a> {
        // Concatenate existing atoms, atoms taken from lang2 will need to be
        // update with an offset to their id's
        let lang2_offset = lang1.atoms.len();
        let mut atoms = Vec::new();
        atoms.extend(&lang1.atoms);
        atoms.extend(&lang2.atoms);

        // Enumerate first atoms
        let mut p = Vec::new();
        p.extend(&lang1.p);

        if lang1.g {
            p.extend(lang2.p.iter().map(|x| x + lang2_offset));
        }

        // Enumerate last atoms
        let mut d = Vec::new();
        d.extend(lang2.d.iter().map(|x| x + lang2_offset));

        if lang2.g {
            d.extend(&lang1.d);
        }

        // Enumerate factors
        let mut f = Vec::new();
        f.extend(&lang1.f);
        f.extend(
            lang2
                .f
                .iter()
                .map(|(x, y)| (x + lang2_offset, y + lang2_offset)),
        );

        for x in &lang1.d {
            for y in &lang2.p {
                f.push((*x, *y + lang2_offset));
            }
        }

        LocalLang {
            atoms,
            p,
            d,
            f,
            g: lang1.g && lang2.g,
        }
    }

    /// Return a local language containing words from the first or the second
    /// input languages.
    fn alternation(lang1: &LocalLang<'a>, lang2: &LocalLang<'a>) -> LocalLang<'a> {
        // Concatenate existing atoms, atoms taken from lang2 will need to be
        // update with an offset to their id's
        let lang2_offset = lang1.atoms.len();
        let mut atoms = Vec::new();
        atoms.extend(&lang1.atoms);
        atoms.extend(&lang2.atoms);

        // Enumerate first atoms
        let mut p = Vec::new();
        p.extend(&lang1.p);
        p.extend(lang2.p.iter().map(|x| x + lang2_offset));

        // Enumerate last atoms
        let mut d = Vec::new();
        d.extend(lang2.d.iter().map(|x| x + lang2_offset));
        d.extend(&lang1.d);

        // Enumerate factors
        let mut f = Vec::new();
        f.extend(&lang1.f);
        f.extend(
            lang2
                .f
                .iter()
                .map(|(x, y)| (x + lang2_offset, y + lang2_offset)),
        );

        LocalLang {
            atoms,
            p,
            d,
            f,
            g: lang1.g || lang2.g,
        }
    }

    /// Return a local language containing the empty word and the input
    /// language.
    fn optional(lang: &LocalLang<'a>) -> LocalLang<'a> {
        LocalLang::alternation(&LocalLang::epsilon(), &lang)
    }

    /// Return a local language containing words made of one or more
    /// repetitions of words of the input language.
    fn closure(lang: &LocalLang<'a>) -> LocalLang<'a> {
        // Atom list, first atoms, last atoms and the belonging of epsilon are
        // unchanged, only factors need to be incremented.
        let mut f = lang.f.clone();

        for x in &lang.d {
            for y in &lang.p {
                f.push((*x, *y));
            }
        }

        LocalLang {
            atoms: lang.atoms.clone(),
            p: lang.p.clone(),
            d: lang.d.clone(),
            f: f,
            g: lang.g,
        }
    }

    /// Return a local language containing words made of an interval-defined
    /// count of words from the input language.
    fn repetition(lang: LocalLang<'a>, range: &hir::RepetitionRange) -> LocalLang<'a> {
        let (min, max) = match range {
            hir::RepetitionRange::Exactly(n) => (*n, Some(*n)),
            hir::RepetitionRange::AtLeast(n) => (*n, None),
            hir::RepetitionRange::Bounded(m, n) => (*m, Some(*n)),
        };

        let mut result = LocalLang::epsilon();

        for i in 0..min {
            if i == min - 1 && max == None {
                result = LocalLang::concatenation(&result, &LocalLang::closure(&lang));
            } else {
                result = LocalLang::concatenation(&result, &lang);
            }
        }

        if let Some(max) = max {
            let mut optionals = LocalLang::empty();

            for _ in min..max {
                optionals = LocalLang::optional(&LocalLang::concatenation(&lang, &optionals));
            }

            result = LocalLang::concatenation(&result, &optionals);
        }

        result
    }
}
