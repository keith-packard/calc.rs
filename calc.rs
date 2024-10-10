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

/// Turn this on to get tracing.
const TRACE: bool = false;

/// Terminals, but also non-terminals and actions.
#[derive(PartialEq, Hash, Eq, Clone, Copy, Debug)]
#[rustfmt::skip]
enum Token {
    // Terminals
    TOp, TCp,
    TNumber,
    TPlus, TMinus, TTimes, TDivide,
    TNl, TEnd,
    // Non-terminals
    Start,
    Expr, ExprP,
    Term, TermP,
    Fact,
    Line,
    // Actions
    Negate,
    Add, Subtract, Times, Divide,
    Push, Print,
}
use Token::*;

/// Get a single character from standard input.
fn getc() -> char {
    let mut c: [u8; 1] = [0];
    let _ = std::io::stdin().read(&mut c);
    c[0] as char
}

/// Return the next token, with an optional value.
fn lex(c: &mut char) -> (Token, Option<f64>) {
    let mut val: f64 = 0.0;
    if *c == '\0' {
        *c = getc();
    }
    loop {
        let token = match *c {
            ' ' | '\t' => {
                *c = getc();
                continue;
            }
            '\0' => TEnd,
            '\n' => TNl,
            '0'..='9' => loop {
                val = val * 10.0 + (*c as u32 - '0' as u32) as f64;
                *c = getc();
                #[allow(clippy::manual_is_ascii_check)]
                if !matches!(*c, '0'..='9') {
                    return (TNumber, Some(val));
                }
            },
            '+' => TPlus,
            '-' => TMinus,
            '*' => TTimes,
            '/' => TDivide,
            '(' => TOp,
            ')' => TCp,
            _ => {
                println!("Invalid char {}", *c);
                *c = getc();
                continue;
            }
        };
        *c = '\0';
        return (token, None);
    }
}

fn main() -> ExitCode {
    let table: HashMap<(Token, Token), Vec<Token>> = HashMap::from([
        ((TCp, ExprP), vec![]),
        ((TCp, TermP), vec![]),
        ((TDivide, TermP), vec![TDivide, Fact, Divide, TermP]),
        ((TEnd, Start), vec![]),
        ((TMinus, Expr), vec![Term, ExprP]),
        ((TMinus, ExprP), vec![TMinus, Term, Subtract, ExprP]),
        ((TMinus, Fact), vec![TMinus, Fact, Negate]),
        ((TMinus, Line), vec![Expr, Print, TNl]),
        ((TMinus, Start), vec![Line, Start]),
        ((TMinus, Term), vec![Fact, TermP]),
        ((TMinus, TermP), vec![]),
        ((TNl, ExprP), vec![]),
        ((TNl, Line), vec![TNl]),
        ((TNl, Start), vec![Line, Start]),
        ((TNl, TermP), vec![]),
        ((TNumber, Expr), vec![Term, ExprP]),
        ((TNumber, Fact), vec![TNumber, Push]),
        ((TNumber, Line), vec![Expr, Print, TNl]),
        ((TNumber, Start), vec![Line, Start]),
        ((TNumber, Term), vec![Fact, TermP]),
        ((TOp, Expr), vec![Term, ExprP]),
        ((TOp, Fact), vec![TOp, Expr, TCp]),
        ((TOp, Line), vec![Expr, Print, TNl]),
        ((TOp, Start), vec![Line, Start]),
        ((TOp, Term), vec![Fact, TermP]),
        ((TPlus, ExprP), vec![TPlus, Term, Add, ExprP]),
        ((TPlus, TermP), vec![]),
        ((TTimes, TermP), vec![TTimes, Fact, Times, TermP]),
    ]);

    let mut value_stack: Vec<f64> = Vec::new();
    let mut stack: Vec<Token> = Vec::new();
    stack.push(Start);
    let mut c: char = '\0';
    let (mut token, mut token_value) = lex(&mut c);
    let mut val = 0.0;
    loop {
        if TRACE {
            print!("token {:#?}, {:#?} stack", token, token_value);
            for v in &stack {
                print!(" {:#?}", v);
            }
            println!();
        }
        match stack.pop() {
            Some(current_state) => match current_state {
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
                #[rustfmt::skip]
                TOp | TCp | TNumber | TPlus | TMinus | TTimes | TDivide | TNl | TEnd => {
                    if current_state != token {
                        println!("syntax error");
                        return ExitCode::from(1);
                    }
                    if current_state == TNumber {
                        val = token_value.unwrap();
                    } else {
                        assert_eq!(token_value, None);
                    }
                    (token, token_value) = lex(&mut c);
                }
                _ => {
                    if !table.contains_key(&(token, current_state)) {
                        println!("syntax error");
                        return ExitCode::from(1);
                    }
                    let _new_bits = &table[&(token, current_state)];
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
