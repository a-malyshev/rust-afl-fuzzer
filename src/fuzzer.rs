use std::time::Instant;
use std::process::Command;
use std::collections::HashSet;
use std::fmt;
use crate::mutator::*;
use crate::scheduler::*;
use std::str;
use std::process::Stdio;
use std::io::Write;


#[derive(Debug)]
pub struct Coverage {
    unique_lines: HashSet<i32>,
    hitted_lines: usize,
    unique_hits: i32,
    reachable_lines: usize,
}

impl fmt::Display for Coverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut sorted_hits: Vec<i32> = self.unique_lines.clone().into_iter().collect();
        sorted_hits.sort();
        write!(f, "{}\n{:?}", self.unique_hits, sorted_hits)
    }
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

pub struct Fuzzer <'f> {
    seeds: Vec<String>,
    pub population: Vec<Seed>,
    print_stats: bool,
    mutator: &'f mut Mutator,
    scheduler: &'f mut Scheduler,
    inputs: Vec<String>,
}

impl <'f>Fuzzer<'f> {
    pub fn new(seeds: Vec<String>, mutator: &'f mut Mutator, scheduler: &'f mut Scheduler, print_stats: bool) -> Fuzzer<'f> {
        Fuzzer {
            seeds,
            population: vec![],
            print_stats,
            mutator,
            scheduler,
            inputs: vec![],
        }
    }

    pub fn fuzz<'a, 's: 'a>(&'a mut self) -> String {
        let seeds = self.seeds.clone();
        self.fill_population(seeds);
        let mut population = self.population.clone();
        let candidate = self.scheduler.select_next(&mut population);

        let res = self.mutator.mutate(candidate.value.clone());
        candidate.value = res.clone();
        self.population = population;
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

    fn reset<'a>(&'a mut self) {
        self.population = self.inputs.clone().into_iter().map(|s| Seed::new(s)).collect();
    }

    pub fn run<'s: 'f>(&'f mut self, dir: &str, cmd_name: &'f str) {
        let time = Instant::now();
        let max_num_iter = 50000;
        let mut total_number_of_lines = 0;
        let mut unique_hits_global: HashSet<i32> = HashSet::new();
        let mut num_exec = 0;
        let mut num_hits = 0;
        let mut non_unique_hits = 0;
        let mut running = true;

        while non_unique_hits <= max_num_iter && running {

            let candidate = self.fuzz();
            debug!("next candidate is {}", candidate);

            debug!("running the program");
            let _result = exec_program(dir, cmd_name, candidate.clone());

            debug!("parsing coverage results");
            let map = parser::parse_coverage(dir);

            debug!("extracting unique hits");
            let cov = parser::get_cov(map, &unique_hits_global);
            num_hits += cov.hitted_lines;
            if num_exec == 0 {
                total_number_of_lines =  cov.reachable_lines;
            }
            
            if cov.unique_hits > 0 {
                non_unique_hits = 0;
                self.update_population(candidate.clone());
                self.inputs.push(candidate.clone());
                self.print_stats(format!("candidate: `{}`", candidate));
                self.print_stats(format!("lines covered: [{}/{}]", cov.hitted_lines, total_number_of_lines));
                self.print_stats(format!("unique hits: {}, hits are: {:?}", cov.unique_hits, cov.unique_lines));
                self.print_stats(format!("--------------------------------------------"));
                if cov.hitted_lines == total_number_of_lines {
                    self.print_stats(format!("all lines are reached. Stopping fuzzing"));
                    running = false;
                }
            } else {
                debug!("population: {:?}", self.population);
                non_unique_hits += 1;
                if non_unique_hits % 5000 == 0 {
                    self.print_stats(format!("{} hits since last unique one", non_unique_hits));
                    if non_unique_hits % 10000 == 0 {
                        self.print_stats(format!("resetting seeds and genereting new ones"));
                        self.reset();
                        self.population.extend(gen_random_strings().into_iter().map(|s| Seed::new(s)));
                    } else if non_unique_hits % max_num_iter == 0 {
                        self.print_stats(format!("after {} various mutations there still is no unique hits, so stopping fuzzing", non_unique_hits));
                        running = false;
                    }
                }
            }

            unique_hits_global.extend(cov.unique_lines);
            num_exec += 1;

            remove_coverage_info(dir, cmd_name);
        }

        self.print_stats(format!("time: {} seconds", time.elapsed().as_secs()));
        self.print_stats(format!("total number of executions: {}", num_exec));
        self.print_stats(format!("inputs that led to unique coverage: {:?}", self.inputs));
        self.print_stats(format!("--------------------------------------------"));
        let mut sorted_hits: Vec<i32> = unique_hits_global.clone().into_iter().collect();
        sorted_hits.sort();
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

    fn print_stats(&mut self, s: String) {
        if self.print_stats {
            println!("{}", s);
        }
    }
}

fn exec_program(dir: &str, cmd_name: &str, input: String) -> String {
    let mut args = vec![];
    if input.len() != 0 { args.push(input.clone()) }

    let mut child = Command::new(get_full_path(dir, cmd_name))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(input.as_bytes()).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    Command::new("gcov")
        .arg(format!("{}.c", cmd_name))
        .current_dir(dir)
        .output()
        .expect("failed to execute gcov");
    let res = output.stdout.to_vec();
    let res: &str = unsafe { str::from_utf8_unchecked(&res) };
    res.to_owned()
}

pub fn get_full_path(path: &str, cmd: &str) -> String {
    format!("{}/{}", path, cmd)
}

fn remove_coverage_info(dir: &str, cmd_name: &str) {
    Command::new("rm")
        .current_dir(dir)
        .arg("-f")
        .arg(format!("{}.c", cmd_name))
        .output()
        .expect("failed to execute gcov");
}

mod parser {
    use super::HashSet;
    use super::Coverage;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::path::Path;
    use lazy_static::lazy_static;
    use regex::Regex;

    pub fn get_cov(map: HashMap<String, String>, unique_hits_global: &HashSet<i32>) -> Coverage {
        let map = get_reachable_lines(map);
        let reachable_lines = map.len() as usize;
        let hitted_lines = get_hitted_lines(map);
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
    fn get_hitted_lines(map: HashMap<String,String>) -> HashMap<i32, i32> {
        get_reachable_lines(map).iter()
            .filter(|(_k,v)| !v.contains("#"))
            .map(|(k,v)| (k.parse().unwrap(), v.parse().unwrap()))
            .collect()
    }

    fn get_reachable_lines(map: HashMap<String,String>) -> HashMap<String,String>{
        map.into_iter()
            .filter(|(_k,v)| !v.contains("-"))
            .collect()
    }

    /// parse coverage of the executed program. Return a map with lines and number of hits
    pub fn parse_coverage(path: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Ok(lines) = read_lines(super::get_full_path(path, "cgi.c.gcov")) {
            for (i, line) in lines.enumerate() {
                if let Ok(s) = line {
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
        }
    }

    fn clean_str(s: &str) -> String {
        let result = s.replace(":", "");
        result.trim_start().to_owned()
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
