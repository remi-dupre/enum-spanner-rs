//  _   _       _ _     _____         _
// | | | |_ __ (_) |_  |_   _|__  ___| |_ ___
// | | | | '_ \| | __|   | |/ _ \/ __| __/ __|
// | |_| | | | | | |_    | |  __/\__ \ |_\__ \
//  \___/|_| |_|_|\__|   |_|\___||___/\__|___/
//

mod unit {
    use super::super::is_match;

    #[test]
    fn wildcard() {
        assert!(is_match(r".", "a"));
        assert!(is_match(r".", "8"));
        assert!(is_match(r".", "\t"));
        assert!(!is_match(r".", ""));
    }

    #[test]
    fn escaped() {
        assert!(is_match(r"\.", "."));
        assert!(is_match(r"\\", "\\"));
        assert!(is_match(r"\.", "."));
        assert!(is_match(r"\t", "\t"));
        assert!(!is_match(r"\\n", "\n"));
        assert!(!is_match(r"\.", "a"));
    }

    #[test]
    fn charclass() {
        assert!(is_match(r"[a-zA-Z0-9]", "a"));
        assert!(is_match(r"[a-zA-Z0-9]", "A"));
        assert!(is_match(r"[a-zA-Z0-9]", "0"));
        assert!(!is_match(r"[a-zA-Z0-9]", "."));

        assert!(is_match(r"[abc]", "a"));
        assert!(!is_match(r"[abc]", "d"));
        assert!(!is_match(r"[.]", "a"));
    }

    #[test]
    fn charclass_complement() {
        assert!(!is_match(r"[^a-zA-Z0-9]", "a"));
        assert!(!is_match(r"[^a-zA-Z0-9]", "A"));
        assert!(!is_match(r"[^a-zA-Z0-9]", "0"));
        assert!(is_match(r"[^a-zA-Z0-9]", "."));

        assert!(!is_match(r"[^abc]", "a"));
        assert!(is_match(r"[^abc]", "d"));
        assert!(is_match(r"[^.]", "a"));
    }

    #[test]
    fn star() {
        assert!(is_match(r"^a*$", ""));
        assert!(is_match(r"^a*$", "aaaaaaaa"));
        assert!(is_match(r"^(foo)*$", "foofoofoo"));
        assert!(!is_match(r"^a*$", "bbbb"));
    }

    #[test]
    fn plus() {
        assert!(is_match(r"^a+$", "aaaaaaaa"));
        assert!(is_match(r"^(foo)+$", "foofoofoo"));
        assert!(!is_match(r"^a+$", ""));
        assert!(!is_match(r"^a+$", "bbbb"));
    }

    #[test]
    fn optional() {
        assert!(is_match(r"^(foo)?$", ""));
        assert!(is_match(r"^foo?$", "fo"));
    }

    #[test]
    fn concatenation() {
        assert!(is_match(r"^a+b+$", "aaabbb"));
        assert!(!is_match(r"^a+b+$", "abab"));
        assert!(!is_match(r"^a+b+$", "aaaa"));
    }

    #[test]
    fn union() {
        assert!(is_match(r"^foo|bar$", "bar"));
        assert!(!is_match(r"^foo|bar$", "foobar"));
    }

    #[test]
    fn repetition() {
        assert!(!is_match(r"^(ab){5}$", &"ab".repeat(4)));
        assert!(is_match(r"^(ab){5}$", &"ab".repeat(5)));
        assert!(!is_match(r"^(ab){5}$", &"ab".repeat(6)));

        assert!(!is_match(r"^(ab){5,}$", &"ab".repeat(4)));
        assert!(is_match(r"^(ab){5,}$", &"ab".repeat(5)));
        assert!(is_match(r"^(ab){5,}$", &"ab".repeat(6)));

        assert!(is_match(r"^(ab){0,5}$", &"ab".repeat(4)));
        assert!(is_match(r"^(ab){0,5}$", &"ab".repeat(5)));
        assert!(!is_match(r"^(ab){0,5}$", &"ab".repeat(6)));

        assert!(!is_match(r"^(ab){4,5}$", &"ab".repeat(3)));
        assert!(is_match(r"^(ab){4,5}$", &"ab".repeat(4)));
        assert!(is_match(r"^(ab){4,5}$", &"ab".repeat(5)));
        assert!(!is_match(r"^(ab){4,5}$", &"ab".repeat(6)));
    }

    #[test]
    fn begin_token() {
        assert!(is_match(r"^foo", "foobar"));
        assert!(is_match(r"bar", "foobar"));
        assert!(!is_match(r"^bar", "foobar"));
    }

    #[test]
    fn end_token() {
        assert!(is_match(r"bar$", "foobar"));
        assert!(is_match(r"foo", "foobar"));
        assert!(!is_match(r"foo$", "foobar"));
    }
}

//  _____                           _
// | ____|_  ____ _ _ __ ___  _ __ | | ___  ___
// |  _| \ \/ / _` | '_ ` _ \| '_ \| |/ _ \/ __|
// | |___ >  < (_| | | | | | | |_) | |  __/\__ \
// |_____/_/\_\__,_|_| |_| |_| .__/|_|\___||___/
//                           |_|

mod examples {
    use std::collections::HashSet;

    use super::super::super::automaton::Automaton;
    use super::super::mapping::{naive, Mapping};
    use super::super::{compile, iter_matches};

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
            for mapping in iter_matches(&regex, text) {
                ret.insert(mapping);
            }
        }

        ret
    }

    #[test]
    fn block_a() {
        let regex = compile(r"^(.*[^a])?(?P<block_a>a+)([^a].*)?$");
        let texts = ["a", "aaaaaaaaaaaaa", "bbbabb", "aaaabbaaababbbb"];

        assert_eq!(
            naive_results(&regex, texts.iter().map(|&x| x)),
            default_results(&regex, texts.iter().map(|&x| x))
        );
    }

    #[test]
    fn sep_email() {
        let regex = compile(r"\w+@\w+");
        let texts = ["a bba a@b b@a aaa@bab abbababaa@@@babbabb"];

        assert_eq!(
            naive_results(&regex, texts.iter().map(|&x| x)),
            default_results(&regex, texts.iter().map(|&x| x))
        );
    }

    #[test]
    fn substrings() {
        let regex = compile(r".*");
        let texts = ["abcdefghijklmnopqrstuvwxyz"];

        assert_eq!(
            naive_results(&regex, texts.iter().map(|&x| x)),
            default_results(&regex, texts.iter().map(|&x| x))
        );
    }

    #[test]
    fn ordered_blocks() {
        let regex =
            compile(r"^(.*[^a])?(?P<block_a>a+)([^a].*[^b]|[^ab])?(?P<block_b>b+)([^b].*)?$");
        let texts = ["ab", "aaaabbbb", "bbbaaababaaaaaabbbbabbbababbababbabb"];

        assert_eq!(
            naive_results(&regex, texts.iter().map(|&x| x)),
            default_results(&regex, texts.iter().map(|&x| x))
        );
    }

    #[test]
    fn mixed_emails() {
        let regex = compile(r"(?P<login>\w+(\.\w+)*)@(?P<server>\w+\.\w+)");
        let texts = ["aaaa@aaa.aa", "aa@aa a@a.a@a.a.a@a.a.a.a@a.a.a.a.a"];

        assert_eq!(
            naive_results(&regex, texts.iter().map(|&x| x)),
            default_results(&regex, texts.iter().map(|&x| x))
        );
    }
}
