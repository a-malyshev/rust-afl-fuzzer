#[macro_use] extern crate log;
extern crate env_logger;

mod mutator;
mod fuzzer;
mod scheduler;

use std::env;
use mutator::Mutator;
use fuzzer::*;
use scheduler::*;

fn main() {
    env_logger::init();
    let show_enabled = !env::args()
                .find(|arg| arg.eq("-show"))
                .is_none();
    //let mut mutator = Mutator::new("http://nytimes/globalnews");
    let mut mutator = Mutator::new();
    let mut scheduler = Scheduler::new();

    let mut fuzzer = Fuzzer::new(vec!["http://nytimes/globalnews".to_owned(), "42".to_owned()], &mut mutator, &mut scheduler, show_enabled);

    fuzzer.run("cgi");
}
