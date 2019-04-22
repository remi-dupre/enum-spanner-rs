mod automaton;
mod mapping;
mod regex;

fn main() {
    let regex = "a";
    let automaton = regex::parse(regex);

    println!(
        "The automaton has {} states for {} transitions",
        automaton.nb_states(),
        automaton.nb_transitions()
    );
    // println!("{:?} -> {:?}", regex, automaton);
}
