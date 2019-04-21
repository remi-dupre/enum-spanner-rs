mod automata;
mod glushkov;
mod mapping;

fn main() {
    let regex = "(?P<x>a{0,10})b";
    println!("{:?} -> {:?}", regex, automata::Automata::from_regex(regex));
}
