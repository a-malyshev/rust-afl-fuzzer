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

static mut show_enabled: bool = false;

#[derive(Debug)]
struct Coverage {
    hitted_lines: i32,
    unique_hits: HashSet<i32>,
}

fn main() {
    env_logger::init();
    unsafe { 
        show_enabled = !env::args()
                .find(|arg| arg.eq("-show"))
                .is_none();
    }

    let total_number_of_lines;
    let unique_hits = HashSet::new();
    let num_exec = 0;

    let output = Command::new(get_command_name("cgi"))
        .arg("hey+world")
        //.arg("/home/alex/Documents/dev/rust/cse867/hw1/inputs/empty")
        .output()
        .expect("failed to execute cgi");

    let map = get_line_hits_mapping();
    trace!("line - hits mapping is {:?}", map);
    total_number_of_lines =  map.len();
    print_stats(format!("total number of lines: {}", total_number_of_lines));

    

    let res = output.stdout.to_vec();
    let res: &str = str::from_utf8(&res).unwrap();
    info!("here it is {:?}", res);
}

fn get_command_name(cmd: &str) -> String {
    format!("/home/alex/Documents/dev/rust/cse867/hw1/test-app/{}", cmd)
}

fn get_cov(map: HashMap<&str, &str>, unique_hits_global: HashSet<i32>) -> Coverage {
    let unique_hits = 0;

    Coverage {
        hitted_lines: get_hitted_lines(map).values().into_iter().sum(),
        unique_hits: get_hitted_lines(map).keys()
            .filter(|stmt| !unique_hits_global.contains(stmt))
            .collect()
    }
}

/// return hashmap with line number as a key and number of hits as a value.
/// From here, it is easy to calc total number of lines in the program,
/// number of hits for each line and also we can compare how many unique hits were covered.
                        // todo: replace with wraper type
fn get_hitted_lines(map: HashMap<&str,&str>) -> HashMap<i32, i32> {
    map.into_iter() .filter(|(k,v)| !v.contains("#"))
        .filter(|(k,v)| !v.contains("-"))
        .map(|(k,v)| (k.parse().unwrap(), v.parse().unwrap()))
        .collect()
}

fn get_line_hits_mapping() -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(lines) = read_lines("/home/alex/Documents/dev/rust/cse867/hw1/test-app/cgi.c.gcov") {
        for (i, line) in lines.enumerate() {
            if let Ok(s) = line {
                //todo: add debugging
                debug!("original:\n{}: {}", i, s);
                if let Some((l, hits)) = extract_hits_and_line_number(s.as_str()) {
                    debug!("parsed:\nline number:{}; hits:{}", l, hits);
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
    if unsafe {show_enabled} {
        println!("{}", s);
    }
}

#[test]
fn correct_hits() {
    let map = get_line_hits_mapping();
    let total_hits: i32 = map.values().into_iter()
        .filter(|s| !s.contains("-"))
        .filter(|s| !s.contains("#"))
        .map(|s| s.parse::<i32>().unwrap())
        .sum();
    assert_eq!(total_hits, 594);
}

#[test]
fn correct_total_number_of_lines() {
    let map = get_line_hits_mapping();
    assert_eq!(map.len(), 86);
}
