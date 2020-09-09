use crate::fuzzer::Seed;
use rand::Rng;

pub struct Scheduler {
}

impl Scheduler {

    pub fn new() -> Scheduler {
        Scheduler {
        }
    }

    pub fn select_next<'a>(&'a self, population: &'a mut Vec<Seed>) -> &'a mut Seed {
        let len = population.len();
        if rand::random() {
            population.iter_mut().last().unwrap()
        } else {
            population.iter_mut().skip(rand::thread_rng().gen_range(0,len)).find(|_| true).unwrap()
        }
    }
}


#[test]
fn correct_select() {
    let scheduler = Scheduler::new();
    let mut population = vec![Seed::new("".to_owned())];
    let candidate = scheduler.select_next(&mut population);
    assert_eq!(candidate.value, "".to_owned());
}
