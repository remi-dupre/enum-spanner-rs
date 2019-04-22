mod automata;
mod glushkov;
mod mapping;
mod parse;

fn main() {
    let regex = "a{0,10000}";
    let automaton = automata::Automata::from_regex(regex);
    println!(
        "The automaton has {} states for {} transitions",
        automaton.nb_states(),
        automaton.nb_transitions()
    );
    // println!("{:?} -> {:?}", regex, automaton);
}
