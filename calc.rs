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
use std::io::Read;
use std::process::ExitCode;

const TRACE: bool = false;

trait MakeToken {
    fn make_token(self) -> Token;
}

#[derive(PartialEq, Hash, Eq, Clone, Copy, Debug)]
enum ETerminal {
    OP,
    CP,
    NUMBER,
    PLUS,
    MINUS,
    TIMES,
    DIVIDE,
    NL,
    END,
}
use ETerminal::*;

impl MakeToken for ETerminal {
    fn make_token(self) -> Token { Terminal(self) }
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
    fn make_token(self) -> Token { NonTerminal(self) }
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
    fn make_token(self) -> Token { Action(self) }
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
    ( $( $x:expr ), * ) => {
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
    return c[0] as char;
}

fn lex(c: &mut char) -> (ETerminal, f64) {
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
                        return (NUMBER, val);
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
        return (terminal, val);
    }
}

fn main() -> ExitCode {
    let table: HashMap<(ETerminal, ENonTerminal), Vec<Token>> = HashMap::from([
        ((CP, ExprP), token_vec![]),
        ((CP, TermP), token_vec![]),
        (
            (DIVIDE, TermP),
            token_vec![DIVIDE, Fact, Divide, TermP],
        ),
        ((END, Start), token_vec![]),
        ((MINUS, Expr), token_vec![Term, ExprP]),
        (
            (MINUS, ExprP),
            token_vec![MINUS, Term, Subtract, ExprP],
        ),
        (
            (MINUS, Fact),
            token_vec![MINUS, Fact, Negate],
        ),
        (
            (MINUS, Line),
            token_vec![Expr, Print, NL],
        ),
        (
            (MINUS, Start),
            token_vec![Line, Start],
        ),
        ((MINUS, Term), token_vec![Fact, TermP]),
        ((MINUS, TermP), token_vec![]),
        ((NL, ExprP), token_vec![]),
        ((NL, Line), token_vec![NL]),
        ((NL, Start), token_vec![Line, Start]),
        ((NL, TermP), token_vec![]),
        (
            (NUMBER, Expr),
            token_vec![Term, ExprP],
        ),
        (
            (NUMBER, Fact),
            token_vec![NUMBER, Push],
        ),
        (
            (NUMBER, Line),
            token_vec![Expr, Print, NL],
        ),
        (
            (NUMBER, Start),
            token_vec![Line, Start],
        ),
        (
            (NUMBER, Term),
            token_vec![Fact, TermP],
        ),
        ((OP, Expr), token_vec![Term, ExprP]),
        (
            (OP, Fact),
            token_vec![OP, Expr, CP],
        ),
        (
            (OP, Line),
            token_vec![Expr, Print, NL],
        ),
        ((OP, Start), token_vec![Line, Start]),
        ((OP, Term), token_vec![Fact, TermP]),
        (
            (PLUS, ExprP),
            token_vec![PLUS, Term, Add, ExprP],
        ),
        ((PLUS, TermP), token_vec![]),
        (
            (TIMES, TermP),
            token_vec![TIMES, Fact, Times, TermP],
        ),
    ]);

    let mut value_stack: Vec<f64> = Vec::new();
    let mut stack: Vec<Token> = Vec::new();
    stack.push(Start.make_token());
    let mut c: char = '\0';
    let mut token = lex(&mut c);
    let mut val = 0.0;
    loop {
        if TRACE {
            print!("token {:#?}, {} stack", token.0, token.1);
            for v in &stack {
                print!(" {:#?}", v);
            }
            println!("");
        }
        match stack.pop() {
            Some(current_state) => match current_state {
                Action(action) =>
                    match action {
                        Negate => {
                            let a = value_stack.pop().unwrap();
                            value_stack.push(-a);
                        }
                        Add => {
                            let b = value_stack.pop().unwrap();
                            let a = value_stack.pop().unwrap();
                            value_stack.push(a + b);
                        }
                        Subtract => {
                            let b = value_stack.pop().unwrap();
                            let a = value_stack.pop().unwrap();
                            value_stack.push(a - b);
                        }
                        Times => {
                            let b = value_stack.pop().unwrap();
                            let a = value_stack.pop().unwrap();
                            value_stack.push(a * b);
                        }
                        Divide => {
                            let b = value_stack.pop().unwrap();
                            let a = value_stack.pop().unwrap();
                            value_stack.push(a / b);
                        }
                        Push => {
                            value_stack.push(val);
                        }
                        Print => {
                            let a = value_stack.pop().unwrap();
                            println!("result = {}", a);
                        }
                    }
                Terminal(terminal) => {
                    if terminal != token.0 {
                        println!("syntax error");
                        return ExitCode::from(1);
                    }
                    if terminal == NUMBER {
                        val = token.1
                    }
                    token = lex(&mut c);
                }
                NonTerminal(non_terminal) => {
                    if !table.contains_key(&(token.0, non_terminal)) {
                        println!("syntax error");
                        return ExitCode::from(1);
                    }
                    let _new_bits = &table[&(token.0, non_terminal)];
                    if TRACE {
                        print!("push");
                    }
                    for v in _new_bits.iter().rev() {
                        if TRACE {
                            print!(" {:#?}", *v);
                        }
                        stack.push(*v)
                    }
                    if TRACE {
                        println!();
                    }
                }
            },
            None => {
                break;
            }
        }
    }
    ExitCode::SUCCESS
}
