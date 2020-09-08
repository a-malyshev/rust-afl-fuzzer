use rand::prelude::*; 
use rand::distributions::{Distribution, Uniform};

pub struct Mutator {
    pub rng: ThreadRng,
    pub inputs: Vec<&'static str>,
}

impl Mutator {
    pub fn new() -> Mutator{
        Mutator { 
            rng: rand::thread_rng(),
            inputs: vec![],
        }
    }

    pub fn mutate<'f, 'a>(&'a mut self, candidate: String) -> String {
        match self.rng.gen_range(1, 4) {
            1 => self.insert_random_char(candidate),
            2 => self.delete_random_char(candidate),
            3 => self.flip_random_char(candidate),
            _ => panic!("will not happen")
        }
    }

    fn insert_random_char<'f, 'a>(&'a mut self, s: String) -> String {
        let between = Uniform::new_inclusive(32u8,126u8);
        let c: char = between.sample(&mut self.rng) as char;
        let position = if s.len() == 0 { 0 } else { self.rng.gen_range(0, s.len()) };
        debug!("inserting random char `{}:{}` into {}", position, c, s);
        let mut res_str = s.to_owned();
        res_str.insert(position, c);
        res_str
    }
    
    fn delete_random_char<'f, 'a>(&'a mut self, s: String) ->  String {
        if s.len() == 0 {
            return self.insert_random_char(s);
        }
        let position: usize = self.rng.gen_range(0, s.len());
        debug!("deleting random char at position `{}` from {}", position, s);
        let mut res_str = s.to_owned();
        res_str.remove(position);
        res_str
    }

    fn flip_random_char<'f, 'a>(&'a mut self, s: String) -> String {
        if s.len() == 0 {
            return self.insert_random_char(s);
        }
        let pos1: usize = self.rng.gen_range(0, s.len());
        let c1 = s.get(pos1..(pos1+1)).unwrap();
        let pos2: usize = self.rng.gen_range(0, s.len());
        let c2 = s.get(pos2..(pos2+1)).unwrap();
        debug!("flipping random chars ({}:{} with {}:{}) in {}", pos1, c1, pos2, c2, s);
        let mut res_str = s.to_owned();
        res_str.replace_range(pos1..(pos1+1), c2);
        res_str.replace_range(pos2..(pos2+1), c1);
        trace!("flipped str {}", res_str);
        res_str    }
            
}

#[test]
fn test() {
    let mut or_str = "string".to_owned();
    or_str.replace_range(1..2, "b");
    assert_eq!(or_str, "sbring".to_owned());
}

#[test]
fn insert_char() {
    let original_str: String = "string".to_owned();
    let s = Mutator::new().insert_random_char(original_str.clone());
    assert_eq!(s.len(), 7);
    assert_ne!(s, original_str);
}

#[test]
fn insert_char_into_empty_str() {
    let original_str: String = "".to_owned();
    let s = Mutator::new().insert_random_char(original_str.clone());
    assert_eq!(s.len(), 1);
}

#[test]
fn delete_char() {
    let original_str: String = "string".to_owned();
    let s = Mutator::new().delete_random_char(original_str.clone());
    assert_eq!(s.len(), 5);
}

#[test]
fn flip_chars() {
    let original_str: String = "string".to_owned();
    let s = Mutator::new().flip_random_char(original_str.clone());
    assert_eq!(s.len(), 6);
    assert_ne!(s, original_str);
}
