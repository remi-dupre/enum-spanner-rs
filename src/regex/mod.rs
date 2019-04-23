mod glushkov;
mod parse;

use super::automaton::Automaton;
use super::mapping;

pub fn compile(regex: &str) -> Automaton {
    let hir = parse::Hir::from_regex(regex);
    glushkov::LocalLang::from_hir(hir, 0).into_automaton()
}

pub fn is_match(regex: &str, text: &str) -> bool {
    let act_regex = format!(".*(?P<match>{}).*", regex);
    let automaton = compile(&act_regex[..]);
    let matches = mapping::naive::NaiveEnum::new(&automaton, text);
    matches.iter().next().is_some()
}

#[cfg(test)]
mod tests;
