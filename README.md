Constant-Delay Enumeration for Nondeterministic Document Spanners
=================================================================

**This implementation is no longer maintained. A more recent implementation is
available [here](https://github.com/PoDMR/enum-spanner-rs).**

This tool allows you to find efficiently all matches of a regular expression in
a string, i.e., find all contiguous substrings of the string that satisfy the
regular expression (including overlapping substrings).

It is the reimplementation of
[the previous Python prototype](https://github.com/remi-dupre/enum-spanner/).

The tool is being actively developed and has not been thoroughly tested yet.
Use at your own risk.

Requirements
------------

It has been tested and developed using `rustc 1.34` and `cargo 1.34`. It will
not work with older Rust versions shipped by some Linux distributions, e.g.,
with version 1.32. You can check your rust version with `rustc --version`, and
install manually a more recent Rust version from the [Rust
website](https://www.rust-lang.org/learn/get-started).

Specific library requirements can be found in *Cargo.toml* and *Cargo.lock*.

Usage
-----

The quickest way is to run the program through Cargo.

```bash
# Display all occurences of a pattern (regexp) in a file
cargo run --release -- [regexp] [file]
cat [file] | cargo run --release -- [regexp]

# For instance, this example will match 'aa@aa', 'aa@a', 'a@aa' and 'a@a'
echo "aa@aa" | cargo run --release -- ".+@.+"

# List optional parameters
cargo run -- --help

# Run unit tests
cargo test
```

The matches displayed correspond to all distincts substrings of the text that
match the given pattern. If the pattern contains named groups, the tool will
output one match for each possible assignment of the groups.

### Named groups

You can define named groups as follows: `(?P<group_a>a+)(?P<group_b>b+)`. This
example will extract any group of a's followed by a group of b's.

The group named `match` has a special behaviour, it can be used to match only
the part captured by this group. For example:

 - `(?P<match>\w+)@\w+` will enumerate the left parts of any feasible email
   address
 - `^.*(?P<match>\w+@\w+).*$` is equivalent to `\w+@\w+`

Supported Syntax for Regular Expressions
----------------------------------------

The tool supports the same syntax as the Rust's regex crate, which is specified
[here](https://docs.rs/regex/1.1.6/regex/#syntax), except for **anchors, which
are not implemented yet**.

Underlying Algorithm
--------------------

The algorithm used by this tool is described in the research paper
*[Constant-Delay Enumeration for Nondeterministic Document
Spanners](https://arxiv.org/abs/1807.09320)*, by [Amarilli](https://a3nm.net/),
[Bourhis](http://cristal.univ-lille.fr/~bourhis/),
[Mengel](http://www.cril.univ-artois.fr/~mengel/) and
[Niewerth](http://www.theoinf.uni-bayreuth.de/en/team/niewerth_matthias/index.php).

It has been presented at the [ICDT'19](http://edbticdt2019.inesc-id.pt/)
conference.

The tool will first compile the regular expression into a non-deterministic
finite automaton, and then apply an *enumeration algorithm*. Specifically, it
will first pre-process the string (without producing any matches), in time
linear in the string and polynomial in the regular expression. After this
pre-computation, the algorithm produces the matches sequentially, with constant
*delay* between each match.
