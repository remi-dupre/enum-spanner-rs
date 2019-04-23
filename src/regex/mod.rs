mod glushkov;
mod parse;

use super::automaton::Automaton;

pub fn parse(regex: &str) -> Automaton {
    let hir = parse::Hir::from_regex(regex);
    glushkov::LocalLang::from_hir(hir, 0).into_automaton()
}
