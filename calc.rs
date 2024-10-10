/*
 * Copyright © 2024 Keith Packard <keithp@keithp.com>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful, but
 * WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA.
 */

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::mem;
use std::process::ExitCode;

const TRACE: bool = true;

trait MakeToken {
    fn make_token(self) -> Token;
}

#[derive(Clone, Copy, Debug)]
enum ETerminal {
    OP,
    CP,
    NUMBER(f64),
    PLUS,
    MINUS,
    TIMES,
    DIVIDE,
    NL,
    END,
}
use ETerminal::*;

impl Hash for ETerminal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
    }
}

impl PartialEq for ETerminal {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl Eq for ETerminal {}

impl MakeToken for ETerminal {
    fn make_token(self) -> Token {
        Terminal(self)
    }
}

#[derive(PartialEq, Hash, Eq, Clone, Copy, Debug)]
enum ENonTerminal {
    Start,
    Expr,
    ExprP,
    Term,
    TermP,
    Fact,
    Line,
}
use ENonTerminal::*;

impl MakeToken for ENonTerminal {
    fn make_token(self) -> Token {
        NonTerminal(self)
    }
}

#[derive(PartialEq, Hash, Eq, Clone, Copy, Debug)]
enum EAction {
    Negate,
    Add,
    Subtract,
    Times,
    Divide,
    Push,
    Print,
}
use EAction::*;

impl MakeToken for EAction {
    fn make_token(self) -> Token {
        Action(self)
    }
}

#[derive(PartialEq, Hash, Eq, Clone, Copy, Debug)]
enum Token {
    Terminal(ETerminal),
    NonTerminal(ENonTerminal),
    Action(EAction),
}

use Token::*;

macro_rules! token_vec {
    () => { Vec::new() };
    ( $( $x:expr ), + ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x.make_token());
            )*
            temp_vec
        }
    };
}

fn getc() -> char {
    let mut c: [u8; 1] = [0];
    let _ = std::io::stdin().read(&mut c);
    c[0] as char
}

fn lex(c: &mut char) -> ETerminal {
    let mut val: f64 = 0.0;
    if *c == '\0' {
        *c = getc();
    }
    loop {
        let terminal = match *c {
            ' ' | '\t' => {
                *c = getc();
                continue;
            }
            '\0' => END,
            '\n' => NL,
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => loop {
                val = val * 10.0 + (*c as u32 - '0' as u32) as f64;
                *c = getc();
                match *c {
                    '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {}
                    _ => {
                        return NUMBER(val);
                    }
                }
            },
            '+' => PLUS,
            '-' => MINUS,
            '*' => TIMES,
            '/' => DIVIDE,
            '(' => OP,
            ')' => CP,
            _ => {
                println!("Invalid char {}", *c);
                *c = getc();
                continue;
            }
        };
        *c = '\0';
        return terminal;
    }
}

struct Values(Vec<f64>);

impl Values {
    fn pop(&mut self) -> f64 {
        match self.0.pop() {
            Some(v) => v,
            None => {
                panic!("Internal error");
            }
        }
    }

    fn push(&mut self, value: f64) {
        self.0.push(value);
    }

    fn new() -> Self {
        Values(Vec::new())
    }
}

fn main() -> ExitCode {
    let table: HashMap<(ETerminal, ENonTerminal), Vec<Token>> = HashMap::from([
        ((CP, ExprP), token_vec![]),
        ((CP, TermP), token_vec![]),
        ((DIVIDE, TermP), token_vec![DIVIDE, Fact, Divide, TermP]),
        ((END, Start), token_vec![]),
        ((MINUS, Expr), token_vec![Term, ExprP]),
        ((MINUS, ExprP), token_vec![MINUS, Term, Subtract, ExprP]),
        ((MINUS, Fact), token_vec![MINUS, Fact, Negate]),
        ((MINUS, Line), token_vec![Expr, Print, NL]),
        ((MINUS, Start), token_vec![Line, Start]),
        ((MINUS, Term), token_vec![Fact, TermP]),
        ((MINUS, TermP), token_vec![]),
        ((NL, ExprP), token_vec![]),
        ((NL, Line), token_vec![NL]),
        ((NL, Start), token_vec![Line, Start]),
        ((NL, TermP), token_vec![]),
        ((NUMBER(0.0), Expr), token_vec![Term, ExprP]),
        ((NUMBER(0.0), Fact), token_vec![NUMBER(0.0), Push]),
        ((NUMBER(0.0), Line), token_vec![Expr, Print, NL]),
        ((NUMBER(0.0), Start), token_vec![Line, Start]),
        ((NUMBER(0.0), Term), token_vec![Fact, TermP]),
        ((OP, Expr), token_vec![Term, ExprP]),
        ((OP, Fact), token_vec![OP, Expr, CP]),
        ((OP, Line), token_vec![Expr, Print, NL]),
        ((OP, Start), token_vec![Line, Start]),
        ((OP, Term), token_vec![Fact, TermP]),
        ((PLUS, ExprP), token_vec![PLUS, Term, Add, ExprP]),
        ((PLUS, TermP), token_vec![]),
        ((TIMES, TermP), token_vec![TIMES, Fact, Times, TermP]),
    ]);

    let mut values = Values::new();
    let mut stack: Vec<Token> = Vec::new();
    stack.push(Start.make_token());
    let mut c: char = '\0';
    let mut lexeme = lex(&mut c);
    let mut prev_lexeme = END;
    loop {
        if TRACE {
            print!("    {:?}:", lexeme);
            for token in &stack {
                print!(" {:?}", token);
            }
            println!();
        }
        match stack.pop() {
            Some(token) => match token {
                Action(action) => {
                    match action {
                        Negate => {
                            let a = values.pop();
                            values.push(-a);
                        }
                        Add => {
                            let b = values.pop();
                            let a = values.pop();
                            values.push(a + b);
                        }
                        Subtract => {
                            let b = values.pop();
                            let a = values.pop();
                            values.push(a - b);
                        }
                        Times => {
                            let b = values.pop();
                            let a = values.pop();
                            values.push(a * b);
                        }
                        Divide => {
                            let b = values.pop();
                            let a = values.pop();
                            values.push(a / b);
                        }
                        Push => {
                            match prev_lexeme {
                                NUMBER(x) => values.push(x),
                                _ => panic!("Invalid state")
                            }
                        }
                        Print => {
                            let a = values.pop();
                            println!("result = {}", a);
                        }
                    }
                    if TRACE {
                        print!("        ");
                        for value in &values.0 {
                            print!(" {}", value);
                        }
                        println!();
                    }
                }
                Terminal(terminal) => {
                    if terminal != lexeme {
                        println!("syntax error");
                        return ExitCode::from(1);
                    }
                    prev_lexeme = lexeme;
                    lexeme = lex(&mut c);
                }
                NonTerminal(non_terminal) => match table.get(&(lexeme, non_terminal)) {
                    Some(tokens) => {
                        for token in tokens.iter().rev() {
                            stack.push(*token)
                        }
                    }
                    None => {
                        println!("syntax error");
                        return ExitCode::from(1);
                    }
                },
            },
            None => {
                break;
            }
        }
    }
    ExitCode::SUCCESS
}
