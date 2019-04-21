use regex_syntax::Parser;
mod glushkov;

fn main() {
    let hir = Parser::new().parse("remi|dupre{5,10}").unwrap();
    let a = glushkov::LocalLang::from_hir(&hir);
    println!("{:?}", a);
}
