mod automaton;
mod mapping;
mod regex;

fn main() {
    let regex = r"(.*\s)?(?P<x>[^\s]+)(\s.*)?\s(?P<y>[^\s]+)(\s.*)?";
    let automaton = regex::compile(regex);

    let text = "salut, ça va !?";

    println!(
        "The automaton has {} states for {} transitions",
        automaton.nb_states(),
        automaton.nb_transitions()
    );

    for x in mapping::naive::NaiveEnum::new(&automaton, &text).iter() {
        println!("{}", x);
    }

    // println!("{:?} -> {:?}", regex, automaton);
}
