// Should I use a HashSet?
use std::collections::{BTreeSet, BTreeMap};
use std::fmt;


#[derive(Debug, Clone, Copy)]
enum MaybeState {
    State(usize),
    Trap,
}

#[derive(Debug)]
pub struct M {
    delta: BTreeMap<(usize, char), Vec<usize>>,
    f: BTreeSet<usize>,
    state: MaybeState,
    previous_states: Vec<MaybeState>,
}


impl M {
    pub fn new(flat_delta: &[(usize, char, usize)], flat_f: &[usize]) -> M {
        let mut delta = BTreeMap::new();
        for &(state, input, next_state) in flat_delta {
            delta
                .entry((state, input))
                .or_insert(vec![])
                .push(next_state);
        }

        let f = flat_f.iter().cloned().collect();


        M {
            delta: delta,
            f: f,
            state: MaybeState::State(0),
            previous_states: vec![],
        }
    }


    pub fn next(&mut self, input: char) -> bool {
        self.previous_states.push(self.state);

        if let MaybeState::State(state) = self.state {
            match self.delta.get(&(state, input)) {
                Some(next_states) => {
                    assert_eq!(next_states.len(), 1, "Expected a single next state (DFA), but found {:?}", next_states);
                    self.state = MaybeState::State(next_states[0]);
                },
                None => {
                    self.state = MaybeState::Trap;
                },
            }
        }

        self.is_accepted()
    }

    pub fn rollback(&mut self) {
        if let Some(prev_state) = self.previous_states.pop() {
            self.state = prev_state;
        }
    }

    pub fn is_accepted(&self) -> bool {
        return match self.state {
            MaybeState::Trap => false,
            MaybeState::State(state) => self.f.contains(&state),
        }
    }

    pub fn is_trapped(&self) -> bool {
        match self.state {
            MaybeState::Trap => true,
            _ => false,
        }
    }

    pub fn reset(&mut self) {
        self.state = MaybeState::State(0);
        self.previous_states = vec![];
    }

    pub fn print_state(&self) {
        println!("state:          {:?}", self.state);
        println!("previous state: {:?}", self.previous_states);
        println!("");
    }
}

impl fmt::Display for M {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "AUTOMATA").unwrap();
        writeln!(f, "========").unwrap();
        writeln!(f, "f: {:?}", self.f).unwrap();
        writeln!(f, "Delta").unwrap();
        for (k, v) in &self.delta {
            writeln!(f, "{:?} -> {:?}", k, v).unwrap();
        }
        writeln!(f, "========")
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn eval_string(m: &mut M, s: &str) {
        for c in s.chars() {
            m.next(c);
            m.print_state();
        }
    }

    fn alphabetic_automata() -> M {
        let lowercase = "abcdefghijklmnopqrstuvwxyz";
        let delta: Vec<_> = lowercase.chars().flat_map(|c| vec![(0, c, 1), (1, c, 1)]).collect();
        M::new(delta.as_slice(), &[1])
    }

    fn numeric_automata() -> M {
        let numbers = "0123456789";
        let delta: Vec<_> = numbers.chars().flat_map(|c| vec![(0, c, 1), (1, c, 1)]).collect();
        M::new(delta.as_slice(), &[1])
    }

    fn string_automata() -> M {
        let lowercase = "abcdefghijklmnopqrstuvwxyz0123456789 '";
        let mut delta = vec![
            (0, '"', 1),
            (1, '"', 2),
        ];
        delta.extend(lowercase.chars().map(|c| (1, c, 1)));
        M::new(delta.as_slice(), &[2])
    }

    #[test]
    fn basic_functionality() {
        let f = [1];
        let delta = [
            (0, 'a', 0),
            (0, 'b', 1),
        ];

        let mut automata = M::new(&delta, &f);
        println!("{}", automata);

        println!("positive case");
        eval_string(&mut automata, "aaaaaab");
        assert_eq!(automata.is_accepted(), true);
        automata.reset();

        println!("negative case");
        eval_string(&mut automata, "abb");
        assert_eq!(automata.is_accepted(), false);
        automata.reset();
    }

    #[test]
    fn rollback() {
        let f = [1];
        let delta = [
            (0, 'a', 0),
            (0, 'b', 1),
        ];

        let mut automata = M::new(&delta, &f);
        println!("{}", automata);

        let s = "aaabb";
        let mut was_accepted = false;
        let mut lexeme = String::new();

        // You can enumerate easily and get the latest index in chars
        // to parse the next lexeme
        for c in s.chars() {
            lexeme.push(c);
            automata.next(c);
            automata.print_state();

            if automata.is_accepted() {
                was_accepted = true;
            }

            if automata.is_trapped() {
                break;
            }
        }

        if was_accepted {
            loop {
                if automata.is_accepted() {
                    break;
                }

                automata.rollback();
                lexeme.pop();
            }
        } else {
            lexeme = String::new();
        }

        println!("LEXEME {:?}", lexeme);
        assert_eq!(lexeme, "aaab");
    }

    #[test]
    fn alphabetic_automata_test() {
        let mut a = alphabetic_automata();

        let inputs = [
            "aaaaaab",
            "lkjasdlkjehasdljhasdljhaskdjh",
            "hello",
            "world",
        ];

        for input in inputs.iter() {
            eval_string(&mut a, input);
            assert_eq!(a.is_accepted(), true);
            a.reset();
        }
    }

    #[test]
    fn string_automata_test() {
        let mut a = string_automata();

        let inputs = [
            "\"aaaaaab\"",
        ];

        for input in inputs.iter() {
            eval_string(&mut a, input);
            assert_eq!(a.is_accepted(), true);
            a.reset();
        }
    }

    #[test]
    fn numeric_automata_test() {
        let mut a = numeric_automata();

        let inputs = [
            "1234",
            "123455677",
            "0123",
        ];

        for input in inputs.iter() {
            eval_string(&mut a, input);
            assert_eq!(a.is_accepted(), true);
            a.reset();
        }
    }

    #[test]
    fn lex_two_types() {
        let a0 = M::new(
            &[
                (0, 'i', 1),
                (1, 'f', 2),
            ],
            &[2]
        );

        let a1 = alphabetic_automata();

        let types = ["IF", "ID"];
        let mut patterns = [a0, a1];

        let s = "ifa";
        //TODO lexeme should be a binary? to allow numbers and strings?
        let mut lexeme = String::new();
        let mut was_accepted = false;

        for c in s.chars() {
            lexeme.push(c);

            let mut trapped = true;
            for p in patterns.iter_mut() {
                p.next(c);
                if p.is_accepted() {
                    was_accepted = true;
                }

                trapped = trapped && p.is_trapped();
            }

            if trapped {
                break;
            }
        }

        for p in patterns.iter() {
            p.print_state();
        }

        let mut category = "NO MATCH";
        if was_accepted {
            let mut end = false;
            loop {
                for (i, p) in patterns.iter_mut().enumerate() {
                    if p.is_accepted() {
                        end = true;
                        category = types[i];
                    } else {
                        p.rollback();
                    }

                }

                if end {
                    break;
                }

                lexeme.pop();
            }
        } else {
            lexeme = String::new();
        }

        println!("CATEGORY {:?}", category);
        println!("LEXEME {:?}", lexeme);
        assert_eq!(lexeme, "ifa");
        assert_eq!(category, "ID");
    }

    #[test]
    fn simple_lexer() {

        fn lex(s: &str) -> (&'static str, String) {
            let a0 = alphabetic_automata();
            let a1 = numeric_automata();
            let a2 = string_automata();
            let a3 = M::new(&[ (0, '(', 1), ], &[1]);
            let a4 = M::new(&[ (0, ')', 1), ], &[1]);
            let a5 = M::new(&[ (0, ' ', 1), ], &[1]);
            let a6 = M::new(&[ (0, '>', 1), ], &[1]);

            let mut patterns = [a0, a1, a2, a3, a4, a5, a6];
            let types = ["ID", "NUMBER", "STRING", "PAROPEN", "PARCLOSE", "SPACE", "OPREL"];




            let mut lexeme = String::new();
            let mut was_accepted = false;

            for c in s.chars() {
                lexeme.push(c);

                let mut trapped = true;
                for p in patterns.iter_mut() {
                    p.next(c);
                    if p.is_accepted() {
                        was_accepted = true;
                    }

                    trapped = trapped && p.is_trapped();
                }

                if trapped {
                    break;
                }
            }

            //for p in patterns.iter() {
                //p.print_state();
            //}

            let mut category = "NO_MATCH";
            if was_accepted {
                let mut end = false;
                loop {
                    for (i, p) in patterns.iter_mut().enumerate() {
                        if p.is_accepted() {
                            end = true;
                            category = types[i];
                        } else {
                            p.rollback();
                        }

                    }

                    if end {
                        break;
                    }

                    lexeme.pop();
                }
            } else {
                lexeme = String::new();
            }

            // In here we could keep a state variable that controls if the next thing
            // should've been a separator or not, so can catch problems such as abc123

            (category, lexeme)
        }

        assert_eq!(lex("hellow x y"), ("ID", "hellow".to_string()));
        assert_eq!(lex("(hellow"), ("PAROPEN", "(".to_string()));
        assert_eq!(lex("123456789 x"), ("NUMBER", "123456789".to_string()));


        let source = "(define (myfn x y) (if (> x y) x y))";
        let mut last_lexed = 0;
        loop {
            if last_lexed >= source.len() {
                break;
            }
            let (category, lexeme) = lex(&source[last_lexed..]);
            println!("{}, {}", category, lexeme);
            last_lexed += lexeme.len();
            if category == "NO_MATCH" {
                panic!("ERROR");
            }
        }
    }
}
