mod glushkov;
mod parse;

use super::automaton::Automaton;
use super::mapping;

pub fn compile(regex: &str) -> Automaton {
    let hir = parse::Hir::from_regex(regex);
    glushkov::LocalLang::from_hir(hir, 0).into_automaton()
}

pub fn is_match(regex: &str, text: &str) -> bool {
    let regex = match regex.as_bytes().first() {
        Some(c) if *c == b'^' => format!("(?P<match>{}", &regex[1..]),
        _ => format!(".*(?P<match>{}", regex),
    };

    let regex = match regex.as_bytes().last() {
        Some(c) if *c == b'$' => format!("{})", &regex[..regex.len() - 1]),
        _ => format!("{}).*", regex),
    };

    let automaton = compile(&regex[..]);
    let matches = mapping::naive::NaiveEnum::new(&automaton, text);
    matches.iter().next().is_some()
}

#[cfg(test)]
mod tests;
