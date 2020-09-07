use std::time::Instant;
use std::process::Command;
use std::str;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt;
use rand::seq::SliceRandom;

use crate::mutator::*;
use crate::scheduler::*;

#[derive(Debug)]
struct Coverage {
    unique_lines: HashSet<i32>,
    hitted_lines: usize,
    unique_hits: i32,
    reachable_lines: usize,
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

pub struct Fuzzer <'f> {
    seeds: Vec<String>,
    pub population: Vec<Seed>,
    print_stats: bool,
    mutator: &'f mut Mutator,
    scheduler: &'f mut Scheduler,
}

#[derive(Clone, Debug)]
pub struct Seed {
    pub value: String,
    pub energy: u32,
}

impl Seed {
    pub fn new(value: String) -> Seed {
        Seed {
            value,
            energy: 0,
        }
    }
}

impl <'f>Fuzzer<'f> {
    pub fn new(seeds: Vec<String>, mutator: &'f mut Mutator, scheduler: &'f mut Scheduler, print_stats: bool) -> Fuzzer<'f> {
        Fuzzer {
            seeds,
            population: vec![],
            print_stats,
            mutator,
            scheduler,
        }
    }

    pub fn fuzz<'a, 's: 'a>(&'a mut self) -> String {
        let seeds = self.seeds.clone();
        self.fill_population(seeds);
        let mut population = self.population.clone();
        let candidate = self.scheduler.select_next(&mut population);

        let res = self.mutator.mutate(candidate);
        res
    }

    fn fill_population<'a>(&'a mut self, seeds: Vec<String>) {
        let empty = self.population.is_empty();
        if empty { 
            for seed in seeds.into_iter() {
                 self.population.push(Seed::new(seed.clone()));
            }
        }
    }
    
    pub fn run<'s: 'f>(&'f mut self, path: &'f str) {
        let time = Instant::now();
        let mut last_unique_time = Instant::now();
        let mut total_number_of_lines = 0;
        let mut unique_hits_global: HashSet<i32> = HashSet::new();
        let mut num_exec = 0;
        let mut num_hits = 0;
        let mut non_unique_hits = 0;

        while non_unique_hits <= 10000 {

            let candidate = self.fuzz();
            debug!("next candidate is {}", candidate);

            debug!("running the program");
            let _result = self.run_program(path, candidate.clone());

            debug!("parsing coverage results");
            let map = self.parse_coverage("replace_on_correct_path");

            debug!("extracting unique hits");
            let cov = self.get_cov(map, &unique_hits_global);
            num_hits += cov.hitted_lines;
            if num_exec == 0 {
                // what is the better way to get number of lines in the program?
                total_number_of_lines =  cov.reachable_lines;
            }
            
            if cov.unique_hits > 0 {
                last_unique_time = Instant::now();
                non_unique_hits = 0;
                self.update_population(candidate.clone());
                self.print_stats(format!("candidate: `{}`", candidate));
                self.print_stats(format!("lines covered: [{}/{}]", cov.hitted_lines, total_number_of_lines));
                self.print_stats(format!("unique hits: {}, hits are: {:?}", cov.unique_hits, cov.unique_lines));
                self.print_stats(format!("--------------------------------------------"));
            } else {
                let now = Instant::now();
                let passed = now.duration_since(last_unique_time).as_secs();
                if passed != 0 && passed % 5 == 0 {
                    last_unique_time = now;
                    self.print_stats(format!("{} sec since last unique hit", passed));
                }
                non_unique_hits += 1;
                if non_unique_hits % 1000 == 0 {
                    self.print_stats(format!("{} hits since last unique one", non_unique_hits));
                }
            }

            unique_hits_global.extend(cov.unique_lines);
            num_exec += 1;

            Command::new("rm")
                .arg("-f")
                .current_dir(&mut self.get_full_path(""))
                .arg("cgi.c")
                .output()
                .expect("failed to execute gcov");
        }

        self.print_stats(format!("time: {} seconds", time.elapsed().as_secs()));
        self.print_stats(format!("total number of executions: {}", num_exec));
        info!("all generated inputs: {:?}", self.population.iter().map(|s| s.value.clone()).collect::<Vec<String>>());
        self.print_stats(format!("--------------------------------------------"));
        let final_cov = Coverage {
            unique_lines: unique_hits_global.clone(),
            hitted_lines: num_hits,
            unique_hits: unique_hits_global.len() as i32,
            reachable_lines: total_number_of_lines
        }; 
        println!("{}", final_cov);
    }

    pub fn update_population(&mut self, candidate: String) {
        let seed = self.population.iter_mut()
            .find(|s| s.value.to_owned().eq(&candidate));

        if let Some(mut s) = seed {
            s.energy += 1;
        } else { 
            let mut seed = Seed::new(candidate);
            seed.energy += 1;
            self.population.push(seed);
        }
        self.population.sort_by(|a, b| a.energy.cmp(&b.energy));
    }

    fn run_program(&mut self, cmd: &str, input: String) -> String {
        let mut args = vec![];
        if input.len() != 0 { args.push(input) }
        let output = Command::new(&mut self.get_full_path(cmd))
            .args(args)
            .output()
            .expect("failed to execute cgi");
        Command::new("gcov")
            .current_dir(&mut self.get_full_path(""))
            .arg(&mut self.get_full_path("cgi.c"))
            .output()
            .expect("failed to execute gcov");
        let res = output.stdout.to_vec();
        let res: &str = unsafe { str::from_utf8_unchecked(&res) };
        res.to_owned()
    }

    fn get_full_path(&mut self, cmd: &str) -> String {
        format!("/home/alex/Documents/dev/rust/cse867/hw1/test-app/{}", cmd)
    }

    fn get_cov(&mut self, map: HashMap<String, String>, unique_hits_global: &HashSet<i32>) -> Coverage {
        let map = self.get_reachable_lines(map);
        let reachable_lines = map.len() as usize;
        let hitted_lines = self.get_hitted_lines(map);
        let unique_lines: HashSet<i32> = hitted_lines
                .keys()
                .filter(|stmt| !unique_hits_global.contains(stmt))
                .cloned()
                .collect();
        let unique_hits: i32 = unique_lines.len() as i32;
        Coverage {
            unique_lines,
            hitted_lines: hitted_lines.len(),
            unique_hits,
            reachable_lines,
        }
    }

    /// return hashmap with line number as a key and number of hits as a value.
    /// From here, it is easy to calc total number of lines in the program,
    /// number of hits for each line and also we can compare how many unique hits were covered.
                            // todo: replace with wraper type
    fn get_hitted_lines(&mut self, map: HashMap<String,String>) -> HashMap<i32, i32> {
        self.get_reachable_lines(map).iter()
            .filter(|(_k,v)| !v.contains("#"))
            .map(|(k,v)| (k.parse().unwrap(), v.parse().unwrap()))
            .collect()
    }

    fn get_reachable_lines(&mut self, map: HashMap<String,String>) -> HashMap<String,String>{
        map.into_iter()
            .filter(|(_k,v)| !v.contains("-"))
            .collect()
    }

    /// parse coverage of the executed program. Return a map with lines and number of hits
    fn parse_coverage(&mut self, _path: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Ok(lines) = self.read_lines("/home/alex/Documents/dev/rust/cse867/hw1/test-app/cgi.c.gcov") {
            for (i, line) in lines.enumerate() {
                if let Ok(s) = line {
                    //todo: add debugging
                    trace!("original: {}: {}\n", i, s);
                    if let Some((l, hits)) = self.extract_hits_and_line_number(s.as_str()) {
                        trace!("parsed: line number:{}; hits:{}\n", l, hits);
                        map.insert(l, hits);
                    }
                }
            }
        };
        map
    }

    ///read lines from the file and return itrator with lines of contents
    fn read_lines<P>(&mut self, filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
        where P: AsRef<Path> {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }

    //todo: improve readability
    fn extract_hits_and_line_number(&mut self, line: &str) -> Option<(String, String)> {
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
            Some((self.clean_str(vals[1]), self.clean_str(vals[0])))
            //Some((clean_str(vals[1]).parse().unwrap(), clean_str(vals[0]).parse().unwrap()))
        }
    }

    //todo: should I return &str and how to do it? what is the best way? why?
    fn clean_str(&mut self, s: &str) -> String {
        let result = s.replace(":", "");
        result.trim_start().to_owned()
    }

    fn print_stats(&mut self, s: String) {
        if self.print_stats {
            println!("{}", s);
        }
    }

}

#[test]
fn correct_fill_population() {
    let mut mutator = Mutator::new();
    let mut scheduler = Scheduler::new(); 
    let seeds = vec!["".to_owned(), "https://web.site".to_owned()];
    let mut fuzzer = Fuzzer::new(seeds.clone(), &mut mutator, &mut scheduler, true);
    fuzzer.fill_population(seeds.clone());
    assert_eq!(seeds.len(), fuzzer.population.len());
    assert_eq!(seeds, fuzzer.population.into_iter().map(|s| s.value).collect::<Vec<String>>());
}


#[test]
fn correct_update_population() {
    let mut mutator = Mutator::new();
    let mut scheduler = Scheduler::new(); 
    let mut fuzzer = Fuzzer::new(vec!["".to_owned(), "https://web.site".to_owned()], &mut mutator, &mut scheduler, true);
    fuzzer.fuzz();
    assert_eq!(fuzzer.population.len(), 2);
    fuzzer.update_population("".to_owned());
    assert_eq!(fuzzer.population.len(), 2);
    assert_eq!(fuzzer.population.last().unwrap().energy, 1);
    fuzzer.update_population("new".to_owned());
    assert_eq!(fuzzer.population.len(), 3);
}

#[test]
fn empty_candidate() {
    let mut mutator = Mutator::new();
    let mut scheduler = Scheduler::new(); 
    let mut fuzzer = Fuzzer::new(vec!["".to_owned()], &mut mutator, &mut scheduler, true);
    fuzzer.run_program("cgi", "".to_owned());
    assert!(true)
}

#[test]
fn remove_file() {
    let mut mutator = Mutator::new();
    let mut scheduler = Scheduler::new(); 
    let mut fuzzer = Fuzzer::new(vec!["".to_owned()], &mut mutator, &mut scheduler, true);
    Command::new("rm")
        .current_dir(fuzzer.get_full_path(""))
        .arg("-f")
        .arg("test")
        .output()
        .expect("failed to rm");
}

#[test]
fn create_cov() {
    let mut mutator = Mutator::new();
    let mut scheduler = Scheduler::new(); 
    let mut fuzzer = Fuzzer::new(vec!["".to_owned()], &mut mutator, &mut scheduler, true);
    Command::new("gcov")
        .current_dir(fuzzer.get_full_path(""))
        .arg("cgi.c")
        .output()
        .expect("failed to execute gcov");
}
