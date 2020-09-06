#[macro_use] extern crate log;
extern crate env_logger;

use std::process::Command;
use std::str;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use lazy_static::lazy_static;
use regex::Regex;
use std::env;
use std::fmt;

mod mutator;
use mutator::mutator::*;

static mut SHOW_ENABLED: bool = false;

#[derive(Debug)]
struct Coverage {
    unique_lines: HashSet<i32>,
    hitted_lines: i32,
    unique_hits: i32,
}

impl fmt::Display for Coverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut unique_hits = String::from("");
        for line in self.unique_lines.iter() {
            let s = format!("test.c:{}, ",line.to_string());
            unique_hits = format!("{}{}", unique_hits, s); 
        }
        unique_hits = unique_hits.trim_end().trim_end_matches(',').to_string();
        write!(f, "{}\n{}", self.unique_hits, unique_hits)
    }
}

fn main() {
    env_logger::init();
    unsafe { 
        SHOW_ENABLED = !env::args()
                .find(|arg| arg.eq("-show"))
                .is_none();
    }

    let mut total_number_of_lines = 0;
    let mut unique_hits_global: HashSet<i32> = HashSet::new();
    let mut num_exec = 0;
    let mut num_hits = 0;
    let mut non_unique_hits = 0;

    

    //let mut mutator = Mutator::new("http://nytimes/globalnews");
    let mut mutator = Mutator::new("");

    while non_unique_hits <= 1000 {
        let candidate = mutator.fuzz();
        debug!("next candidate is {}", candidate);

        debug!("running the program");
        let result = run_program(candidate);

        debug!("parsing coverage results");
        let map = parse_coverage("replace_on_correct_path");
        if num_exec == 0 {
            // what is the better way to get number of lines in the program?
            total_number_of_lines =  map.len();
        }

        debug!("extracting unique hits");
        let cov = get_cov(map, &unique_hits_global);
        num_hits += cov.hitted_lines;
        
        if cov.unique_hits > 0 {
            non_unique_hits = 0;
            print_stats(format!("candidate: `{}`", candidate));
            print_stats(format!("lines covered: [{}/{}]", cov.hitted_lines, total_number_of_lines));
            print_stats(format!("unique hits: {}, hits are: {:?}", cov.unique_hits, cov.unique_lines));
            print_stats(format!("--------------------------------------------"));
        } else {
            non_unique_hits += 1;
            if non_unique_hits % 100 == 0 {
                print_stats(format!("{} hits since last unique one", non_unique_hits));
            }
        }

        unique_hits_global.extend(cov.unique_lines);
        num_exec += 1;

        Command::new("rm")
            .arg("-f")
            .current_dir(get_full_path(""))
            .arg("cgi.c")
            .output()
            .expect("failed to execute gcov");
    }

    print_stats(format!("total number of executions: {}", num_exec));
    debug!("all generated inputs: {:?}", mutator.inputs);
    print_stats(format!("--------------------------------------------"));
    let final_cov = Coverage {
        unique_lines: unique_hits_global.clone(),
        hitted_lines: num_hits,
        unique_hits: unique_hits_global.len() as i32,
    }; 
    println!("{}", final_cov);
}

fn run_program(input: &String) -> String {
    let mut args = vec![];
    if input.len() != 0 { args.push(input) }
    let output = Command::new(get_full_path("cgi"))
        .args(args)
        //.arg("/home/alex/Documents/dev/rust/cse867/hw1/inputs/empty")
        .output()
        .expect("failed to execute cgi");
    Command::new("gcov")
        .current_dir(get_full_path(""))
        .arg(get_full_path("cgi.c"))
        .output()
        .expect("failed to execute gcov");
    let res = output.stdout.to_vec();
    let res: &str = unsafe { str::from_utf8_unchecked(&res) };
    res.to_owned()
}

fn get_full_path(cmd: &str) -> String {
    format!("/home/alex/Documents/dev/rust/cse867/hw1/test-app/{}", cmd)
}

fn get_cov(map: HashMap<String, String>, unique_hits_global: &HashSet<i32>) -> Coverage {
    let hitted_lines = get_hitted_lines(map);
    let unique_lines: HashSet<i32> = hitted_lines
            .keys()
            .filter(|stmt| !unique_hits_global.contains(stmt))
            .cloned()
            .collect();
    let unique_hits: i32 = unique_lines.len() as i32;
    Coverage {
        unique_lines,
        hitted_lines: hitted_lines.len() as i32,
        unique_hits,
    }
}

/// return hashmap with line number as a key and number of hits as a value.
/// From here, it is easy to calc total number of lines in the program,
/// number of hits for each line and also we can compare how many unique hits were covered.
                        // todo: replace with wraper type
fn get_hitted_lines(map: HashMap<String,String>) -> HashMap<i32, i32> {
    map.into_iter() .filter(|(_k,v)| !v.contains("#"))
        .filter(|(_k,v)| !v.contains("-"))
        .map(|(k,v)| (k.parse().unwrap(), v.parse().unwrap()))
        .collect()
}

/// parse coverage of the executed program. Return a map with lines and number of hits
fn parse_coverage(path: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(lines) = read_lines("/home/alex/Documents/dev/rust/cse867/hw1/test-app/cgi.c.gcov") {
        for (i, line) in lines.enumerate() {
            if let Ok(s) = line {
                //todo: add debugging
                trace!("original: {}: {}\n", i, s);
                if let Some((l, hits)) = extract_hits_and_line_number(s.as_str()) {
                    trace!("parsed: line number:{}; hits:{}\n", l, hits);
                    map.insert(l, hits);
                }
            }
        }
    };
    map
}

///read lines from the file and return itrator with lines of contents
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

//todo: improve readability
fn extract_hits_and_line_number(line: &str) -> Option<(String, String)> {
    lazy_static! {
        static ref FUZZ_REGEX : Regex = Regex::new(
                r"[\s].[0-9|#|-]+[:]"
            ).unwrap();
    }
    let vals: Vec<&str> = FUZZ_REGEX.find_iter(line).map(|mat| mat.as_str()).collect();
    trace!("vec: {:?}", vals);
    if vals.len() == 0 {
        None
    } else {
        Some((clean_str(vals[1]), clean_str(vals[0])))
        //Some((clean_str(vals[1]).parse().unwrap(), clean_str(vals[0]).parse().unwrap()))
    }
}

//todo: should I return &str and how to do it? what is the best way? why?
fn clean_str(s: &str) -> String {
    let result = s.replace(":", "");
    result.trim_start().to_owned()
}

fn print_stats(s: String) {
    if unsafe {SHOW_ENABLED} {
        println!("{}", s);
    }
}

#[test]
fn correct_hits() {
    let map = parse_coverage("path");
    let total_hits: i32 = map.values().into_iter()
        .filter(|s| !s.contains("-"))
        .filter(|s| !s.contains("#"))
        .map(|s| s.parse::<i32>().unwrap())
        .sum();
    assert_eq!(total_hits, 594);
}

#[test]
fn correct_total_number_of_lines() {
    let map = parse_coverage("path");
    assert_eq!(map.len(), 86);
}

#[test]
fn empty_candidate() {
    run_program(&"".to_owned());
    assert!(true)
}

#[test]
fn remove_file() {
    Command::new("rm")
        .current_dir(get_full_path(""))
        .arg("-f")
        .arg("test")
        .output()
        .expect("failed to rm");
}

#[test]
fn create_cov() {
    Command::new("gcov")
        .current_dir(get_full_path(""))
        .arg("cgi.c")
        .output()
        .expect("failed to execute gcov");
}
