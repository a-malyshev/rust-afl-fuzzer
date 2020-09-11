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
    let mut mutator = Mutator::new();
    let mut scheduler = Scheduler::new();

    let mut fuzzer = Fuzzer::new(vec!["42".to_owned(), "https://unl.edu".to_owned()], &mut mutator, &mut scheduler, show_enabled);

    let program_dir = if let Some(dir) = env::args().find(|arg| arg.starts_with("-d")) {
        dir.split('=').last().unwrap().to_owned()
    } else {
        env::current_dir().ok().unwrap().into_os_string().into_string().ok().unwrap()
    };

    let mut cmd_name: String = if let Some(cmd) = env::args().find(|arg| arg.ends_with(".c")) {
        cmd 
    } else {
        panic!("name of the testing C program should be supplied (format `program_name.c`)")
    };
    cmd_name.truncate(cmd_name.len()-2);
    fuzzer.run(&program_dir, cmd_name.as_str());
}
