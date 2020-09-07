use rand::seq::SliceRandom;
use crate::fuzzer::Seed;

pub struct Scheduler {
}

impl Scheduler {

    pub fn new() -> Scheduler {
        Scheduler {
        }
    }

    pub fn select_next<'a>(&'a self, population: &'a mut Vec<Seed>) -> String {
        let len = population.len();
        if len < 3 {
            population.iter().find(|_| true).unwrap().value.clone()
        } else {
            let top_3: Vec<Seed> = population.into_iter().skip(len - 3).map(|s| s.clone()).collect();
            top_3.choose(&mut rand::thread_rng()).unwrap().value.clone()
        }
    }
}


#[test]
fn correct_select() {
    let scheduler = Scheduler::new();
    let mut population = vec![Seed::new("".to_owned())];
    let candidate = scheduler.select_next(&mut population);
    assert_eq!(candidate, "".to_owned());
}
