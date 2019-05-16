use std::collections::HashSet;

use super::super::automaton::Automaton;
use super::super::regex;
use super::{naive, Mapping};

/// Build a HashSet collecting results of naive algorithm.
fn naive_results<'a, T>(regex: &'a Automaton, texts: T) -> HashSet<Mapping<'a>>
where
    T: Iterator<Item = &'a str>,
{
    let mut ret = HashSet::new();

    for text in texts {
        for mapping in naive::NaiveEnum::new(&regex, text) {
            ret.insert(mapping);
        }
    }

    ret
}

/// Build a HashSet collecting results of default algorithm.
fn default_results<'a, T>(regex: &'a Automaton, texts: T) -> HashSet<Mapping<'a>>
where
    T: Iterator<Item = &'a str>,
{
    let mut ret = HashSet::new();

    for text in texts {
        let matches = regex::compile_matches(&regex, text);

        for mapping in matches.iter() {
            ret.insert(mapping);
        }
    }

    ret
}

#[test]
fn block_a() {
    let regex = regex::compile(r"^(.*[^a])?(?P<block_a>a+)([^a].*)?$");
    let texts = ["a", "aaaaaaaaaaaaa", "bbbabb", "aaaabbaaababbbb"];

    assert_eq!(
        naive_results(&regex, texts.iter().map(|&x| x)),
        default_results(&regex, texts.iter().map(|&x| x))
    );
}

#[test]
fn sep_email() {
    let regex = regex::compile(r"\w+@\w+");
    let texts = ["a bba a@b b@a aaa@bab abbababaa@@@babbabb"];

    assert_eq!(
        naive_results(&regex, texts.iter().map(|&x| x)),
        default_results(&regex, texts.iter().map(|&x| x))
    );
}

#[test]
fn substrings() {
    let regex = regex::compile(r".*");
    let texts = ["abcdefghijklmnopqrstuvwxyz"];

    assert_eq!(
        naive_results(&regex, texts.iter().map(|&x| x)),
        default_results(&regex, texts.iter().map(|&x| x))
    );
}

#[test]
fn ordered_blocks() {
    let regex =
        regex::compile(r"^(.*[^a])?(?P<block_a>a+)([^a].*[^b]|[^ab])?(?P<block_b>b+)([^b].*)?$");
    let texts = ["ab", "aaaabbbb", "bbbaaababaaaaaabbbbabbbababbababbabb"];

    assert_eq!(
        naive_results(&regex, texts.iter().map(|&x| x)),
        default_results(&regex, texts.iter().map(|&x| x))
    );
}

#[test]
fn mixed_emails() {
    let regex = regex::compile(r"(?P<login>\w+(\.\w+)*)@(?P<server>\w+\.\w+)");
    let texts = ["aaaa@aaa.aa", "aa@aa a@a.a@a.a.a@a.a.a.a@a.a.a.a.a"];

    assert_eq!(
        naive_results(&regex, texts.iter().map(|&x| x)),
        default_results(&regex, texts.iter().map(|&x| x))
    );
}
