//! Implementation of [Thompson's Construction]
//!
//! [Thompson's Construction]: <https://en.wikipedia.org/wiki/Thompson%27s_construction>
//!
//! Resources:
//!
//! <https://swtch.com/~rsc/regexp/regexp1.html>
//!

#![allow(soft_unstable)]

use std::fmt::Debug;

use crate::state::State;
use crate::token::Token;

impl<T> std::ops::Index<State> for Vec<T> {
    type Output = T;

    fn index(&self, index: State) -> &Self::Output {
        &self[index.0]
    }
}

impl<T> std::ops::IndexMut<State> for Vec<T> {
    fn index_mut(&mut self, index: State) -> &mut Self::Output {
        &mut self[index.0]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Transition {
    Char(char, State),
    Split(Option<State>, Option<State>),
    Accept,
}

#[derive(Debug)]
pub struct Nfa {
    /// Each state has it's own row of transitions.
    /// Thus `transitions.len() == num_states`
    pub transitions: Vec<Transition>,
    pub start: State,
    /// Only a single accepting state.
    pub accept: State,
}

impl Default for Nfa {
    fn default() -> Self {
        Self::new()
    }
}

impl Nfa {
    #[must_use]
    pub fn new() -> Self {
        Self {
            transitions: vec![],
            // Is changed when compiled
            accept: State(0),
            // Is changed when compiled
            start: State(0),
        }
    }
}

impl std::ops::Index<State> for Nfa {
    type Output = Transition;

    fn index(&self, index: State) -> &Self::Output {
        &self.transitions[index]
    }
}

impl std::ops::IndexMut<State> for Nfa {
    fn index_mut(&mut self, index: State) -> &mut Self::Output {
        &mut self.transitions[index]
    }
}

impl Nfa {
    #[must_use]
    pub(crate) fn new_char_state(&mut self, c: char) -> State {
        let state = State(self.transitions.len());
        self.transitions.push(Transition::Char(c, state));
        state
    }

    #[must_use]
    pub(crate) fn new_split_state(&mut self, e1: Option<State>, e2: Option<State>) -> State {
        self.transitions.push(Transition::Split(e1, e2));
        State(self.transitions.len() - 1)
    }

    #[must_use]
    pub(crate) fn new_accept_state(&mut self) -> State {
        self.transitions.push(Transition::Accept);
        State(self.transitions.len() - 1)
    }

    fn patch(&mut self, from: &Frag, to: State) {
        for outp in &from.out {
            match &mut self[*outp] {
                Transition::Split(_, s) => {
                    *s = Some(to);
                }
                Transition::Char(_, s) => {
                    *s = to;
                }

                Transition::Accept => panic!(),
            }
        }
    }
}

#[derive(Debug)]
struct Frag {
    start: State,
    out: Vec<State>,
}

impl Nfa {
    /// Compile postfix notation into an NFA.
    ///
    /// # Errors
    ///
    /// Fails if the postfix stack contians '(' or ')' tokens or has invalid syntax.
    pub fn compile<I: Iterator<Item = Token>>(postfix: I) -> Result<Self, String> {
        let mut nfa = Self::new();

        nfa.accept = nfa.new_accept_state();

        let mut stack: Vec<Frag> = vec![];

        for tok in postfix {
            match tok {
                Token::Union => {
                    //  /-> e1 ->
                    // s
                    //  \-> e2 ->
                    let mut e2 = stack.pop().unwrap();
                    let mut e1 = stack.pop().unwrap();
                    let s = nfa.new_split_state(Some(e1.start), Some(e2.start));
                    e1.out.append(&mut e2.out);
                    e1.start = s;
                    stack.push(e1);
                }
                Token::Concat => {
                    // e1 -> e2 ->
                    let e2 = stack.pop().unwrap();
                    let e1 = stack.pop().unwrap();
                    nfa.patch(&e1, e2.start);

                    stack.push(Frag {
                        start: e1.start,
                        out: e2.out,
                    });
                }
                Token::Char(c) => {
                    //   c
                    // s ->
                    let s = nfa.new_char_state(c);
                    stack.push(Frag {
                        start: s,
                        out: vec![s],
                    });
                }
            }
        }

        if let (1, Some(e)) = (stack.len(), stack.pop()) {
            nfa.start = e.start;
            nfa.patch(&e, nfa.accept);
            Ok(nfa)
        } else {
            Err(format!("Some tokens are still on the stack: {stack:?}"))
        }
    }
}

#[derive(Debug)]
struct Step {
    /// The current char in the input string.
    current_char: char,
    /// Number of bytes of the input consumed thus far.
    consumed: usize,
    /// Contains a number for each state.
    /// If `list[state] == step` then we have reached the state already.
    list: Vec<usize>,
    /// The current step.
    current: usize,
}

impl Step {
    #[must_use]
    fn new(num_states: usize) -> Self {
        Self {
            current_char: 0 as char,
            consumed: 0,
            list: (0..num_states).map(|_| 0).collect(),
            current: 1,
        }
    }

    #[must_use]
    fn is_visited(&self, state: State) -> bool {
        self.list[state] == self.current
    }

    fn set_visited(&mut self, state: State) {
        self.list[state] = self.current;
    }

    fn next_step(&mut self, current_char: char) {
        self.current += 1;
        self.current_char = current_char;
        // The char might be more than one byte.
        self.consumed += current_char.len_utf8();
    }
}

impl Nfa {
    fn add_state(&self, step: &mut Step, list: &mut Vec<State>, state: State) -> bool {
        if step.is_visited(state) {
            return false;
        }

        match &self[state] {
            &Transition::Split(e1, e2) => {
                if let Some(e1) = e1 {
                    if self.add_state(step, list, e1) {
                        return true;
                    }
                }

                if let Some(e2) = e2 {
                    if self.add_state(step, list, e2) {
                        return true;
                    }
                }

                false
            }
            Transition::Char(..) | Transition::Accept => {
                step.set_visited(state);
                list.push(state);

                state == self.accept
            }
        }
    }

    /// Step each state in `current_list` with `c`, following any eps-closuers.
    /// Returns `true` if the accepting state has been reached.
    fn step(&self, step: &mut Step, current_list: &Vec<State>, next_list: &mut Vec<State>) -> bool {
        assert!(next_list.is_empty());
        let mut accept = false;

        for state in current_list {
            match &self[*state] {
                &Transition::Char(c, e) => {
                    accept |= c == step.current_char && self.add_state(step, next_list, e);
                }
                Transition::Accept => {
                    accept = true;
                }
                Transition::Split(_, _) => unreachable!(),
            }
        }

        accept
    }
}

impl Nfa {
    /// Check is the start of the input matches the regex.
    /// Returns the length of any match.
    pub fn find(&self, input: &str) -> Option<usize> {
        let mut current_list = Vec::with_capacity(self.transitions.len());
        let mut next_list = Vec::with_capacity(self.transitions.len());

        let mut step = Step::new(self.transitions.len());
        let mut m = None;

        // Follow any eps-closuers at the start
        self.add_state(&mut step, &mut current_list, self.start);

        for c in input.chars() {
            step.next_step(c);

            if self.step(&mut step, &current_list, &mut next_list) {
                m = Some(step.consumed - 1);
            }

            std::mem::swap(&mut current_list, &mut next_list);
            next_list.truncate(0);
        }

        if self.step(&mut step, &current_list, &mut next_list) {
            m = Some(step.consumed);
        }

        m
    }
}

#[cfg(test)]
impl std::fmt::Display for Nfa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", ["Type", "State", "Label", "e1", "e2"].join("\t"))?;

        for (state, transition) in self.transitions.iter().enumerate() {
            let ty = if state == self.start {
                "Start:"
            } else if state == self.accept {
                "Accept:"
            } else {
                ""
            }
            .to_string();

            let mut lab = String::new();
            let mut edge1 = String::new();
            let mut edge2 = String::new();

            match transition {
                Transition::Char(c, e) => {
                    lab = c.to_string();
                    edge1 = e.to_string();
                }
                Transition::Split(e1, e2) => {
                    edge1 = e1.map(|e1| e1.to_string()).unwrap_or(String::new());
                    edge2 = e2.map(|e2| e2.to_string()).unwrap_or(String::new());
                }
                Transition::Accept => {
                    // Covered in `ty` above
                }
            }

            writeln!(f, "{ty}\t{state}\t{lab}\t{edge1}\t{edge2}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Token, Nfa};

    #[test]
    fn matches() {
        let mut tokens = vec!["letfn", "let", "fn", "lefn"]
            .into_iter()
            .map(|x| {
                let mut out = vec![];
                let mut x = x.chars().map(Token::Char);
                out.push(x.next().unwrap());
                for c in x {
                    out.push(c);
                    out.push(Token::Concat);
                }

                out
            })
            .collect::<Vec<_>>();

        let mut other = vec![];

        other.append(&mut tokens.pop().unwrap());

        while let Some(mut tokens) = tokens.pop() {
            other.append(&mut tokens);
            other.push(Token::Union);
        }

        let nfa = Nfa::compile(other.into_iter()).unwrap();
        // panic!("Correct:\n\n{nfa}");

        assert_eq!(None, nfa.find("l"));
        assert_eq!(Some(3), nfa.find("let"));
        assert_eq!(Some(2), nfa.find("fn"));
        assert_eq!(Some(5), nfa.find("letfn"));
        assert_eq!(Some(4), nfa.find("lefn"));
        assert_eq!(Some(2), nfa.find("fnlet"));
        assert_eq!(Some(5), nfa.find("letfnn"));
        assert_eq!(Some(3), nfa.find("letffn"));
    }
}
