use regex_syntax::Parser;
mod automata;
mod glushkov;
mod mapping;

fn main() {
    let hir = Parser::new().parse("(?P<x>.)").unwrap();
    let a = glushkov::LocalLang::from_hir(&hir);
    println!("{:?}", a);

    let v = mapping::Variable::new("x");
    let m = mapping::Marker::Open(v);
    println!("v: {:?}, m: {:?}", v, m);
}
