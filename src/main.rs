mod automata;
mod glushkov;
mod mapping;

use regex_syntax::Parser;

fn main() {
    let regex = "(?P<x>a{0,10})b";
    let hir = Parser::new().parse(regex).unwrap();
    println!("{:?} -> {:?}", regex, automata::Automata::from_hir(&hir));
}
