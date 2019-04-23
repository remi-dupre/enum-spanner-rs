mod automaton;
mod mapping;
mod regex;

fn main() {
    let regex = r"(.*\s)?(?P<x>[^\s]+)(\s.*)?";
    let automaton = regex::parse(regex);

    println!("{:?}", automaton);

    let text = "salut, ça va !?";

    println!(
        "The automaton has {} states for {} transitions",
        automaton.nb_states(),
        automaton.nb_transitions()
    );

    for x in mapping::naive::NaiveEnum::new(&automaton, &text).iter() {
        println!("{:?}", x);
    }

    // println!("{:?} -> {:?}", regex, automaton);
}
