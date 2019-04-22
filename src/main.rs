mod automata;
mod glushkov;
mod mapping;

fn main() {
    let regex = ".*(?P<match>a{0,10000}).*";
    let automaton = automata::Automata::from_regex(regex);
    // println!("{:?} -> {:?}", regex, automaton);
}
