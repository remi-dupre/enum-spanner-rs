use super::is_match;

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
