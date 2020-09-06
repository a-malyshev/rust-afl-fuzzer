
pub mod mutator {
    use rand::prelude::*; 
    use rand::distributions::{Distribution, Uniform};

    pub struct Mutator {
        rng: ThreadRng,
        pub inputs: Vec<String>,
    }

    impl Mutator {
        pub fn new(seed: &'static str) -> Self {
            Mutator { 
                rng: rand::thread_rng(),
                inputs: vec![seed.to_owned()]
            }
        }

        pub fn fuzz(&mut self) -> &String {
            let candidate = self.next_candidate();
            let mutated = match self.rng.gen_range(1, 4) {
                1 => self.insert_random_char(candidate),
                2 => self.delete_random_char(candidate),
                3 => self.flip_random_char(candidate),
                _ => panic!("will not happen")
            };
            self.inputs.push(mutated);
            self.inputs.last().unwrap()
        }

        fn next_candidate(&self) -> String {
            self.inputs.last().unwrap().clone()
        }

        fn insert_random_char(&mut self, mut s: String) -> String {
            let between = Uniform::new_inclusive(32u8,126u8);
            let c: char = between.sample(&mut self.rng) as char;
            let position = if s.len() == 0 { 0 } else { self.rng.gen_range(0, s.len()) };
            debug!("inserting random char `{}:{}` into {}", position, c, s);
            s.insert(position, c);
            s
        }
        
        fn delete_random_char(&mut self, mut s: String) -> String {
            if s.len() == 0 {
                return self.insert_random_char(s);
            }
            let position: usize = self.rng.gen_range(0, s.len());
            debug!("deleting random char at position `{}` from {}", position, s);
            s.remove(position);
            s
        }

        fn flip_random_char(&mut self, s: String) -> String {
            if s.len() == 0 {
                return self.insert_random_char(s);
            }
            let pos1: usize = self.rng.gen_range(0, s.len());
            let c1 = s.get(pos1..(pos1+1)).unwrap();
            let pos2: usize = self.rng.gen_range(0, s.len());
            let c2 = s.get(pos2..(pos2+1)).unwrap();
            debug!("flipping random chars ({}:{} with {}:{}) in {}", pos1, c1, pos2, c2, s);
            let mut s = s.clone();
            s.replace_range(pos1..(pos1+1), c2);
            s.replace_range(pos2..(pos2+1), c1);
            trace!("flipped str {}", s);
            s
        }
                
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
        let s = Mutator::new("seed").insert_random_char(original_str.clone());
        assert_eq!(s.len(), 7);
        assert_ne!(s, original_str);
    }

    #[test]
    fn insert_char_into_empty_str() {
        let original_str: String = "".to_owned();
        let s = Mutator::new("seed").insert_random_char(original_str.clone());
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn delete_char() {
        let original_str: String = "string".to_owned();
        let s = Mutator::new("seed").delete_random_char(original_str.clone());
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn flip_chars() {
        let original_str: String = "string".to_owned();
        let s = Mutator::new("seed").flip_random_char(original_str.clone());
        assert_eq!(s.len(), 6);
        assert_ne!(s, original_str);
    }
}
