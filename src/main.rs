mod automata;
mod glushkov;
mod mapping;

fn main() {
    let regex = ".*(?P<match>a{0,1000}).*";
    let automaton = automata::Automata::from_regex(regex);
    println!(
        "The automaton has {} states for {} transitions",
        automaton.nb_states(),
        automaton.nb_transitions()
    );
    // println!("{:?} -> {:?}", regex, automaton);
}
