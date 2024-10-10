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

#[derive(PartialEq, Hash, Eq, Clone, Copy, Debug)]
enum Token {
    /* Terminals */
    TOp, TCp,
    TNumber,
    TPlus, TMinus, TTimes, TDivide,
    TNl, TEnd,
    /* Non-terminals */
    Start,
    Expr, ExprP,
    Term, TermP,
    Fact,
    Line,
    /* Actions */
    Negate,
    Add, Subtract, Times, Divide,
    Push, Print,
}

fn getc() -> char {
    let mut c: [u8; 1] = [0];
    let _ = std::io::stdin().read(&mut c);
    c[0] as char
}

fn lex(c: &mut char) -> (Token, f64) {
    let mut val: f64 = 0.0;
    if *c == '\0' {
	*c = getc();
    }
    loop {
	let token;
	match *c {
	    ' ' | '\t' => {
		*c = getc();
		continue;
	    },
	    '\0' => {
		token = Token::TEnd;
	    }
	    '\n' => {
		token = Token::TNl;
	    },
	    '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
		loop {
		    val = val * 10.0 + (*c as u32 - '0' as u32) as f64;
		    *c = getc();
		    match *c {
			'0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {}
			_ => { return (Token::TNumber, val); }
		    }
		} 
	    }
	    '+' => {
		token = Token::TPlus;
	    }
	    '-' => {
		token = Token::TMinus;
	    }
	    '*' => {
		token = Token::TTimes;
	    }
	    '/' => {
		token = Token::TDivide;
	    }
	    '(' => {
		token = Token::TOp;
	    }
	    ')' => {
		token = Token::TCp;
	    }
	    _ => {
		println!("Invalid char {}", *c);
		*c = getc();
		continue;
	    }
	}
	*c = '\0';
	return (token, val)
    }
}

fn main() ->ExitCode {
    let table: HashMap<(Token,Token), Vec<Token>> = HashMap::from([
	((Token::TCp, Token::ExprP), vec![]),
	((Token::TCp, Token::TermP), vec![]),
	((Token::TDivide, Token::TermP), vec![Token::TDivide, Token::Fact, Token::Divide, Token::TermP]),
	((Token::TEnd, Token::Start), vec![]),
	((Token::TMinus, Token::Expr), vec![Token::Term, Token::ExprP]),
	((Token::TMinus, Token::ExprP), vec![Token::TMinus, Token::Term, Token::Subtract, Token::ExprP]),
	((Token::TMinus, Token::Fact), vec![Token::TMinus, Token::Fact, Token::Negate]),
	((Token::TMinus, Token::Line), vec![Token::Expr, Token::Print, Token::TNl]),
	((Token::TMinus, Token::Start), vec![Token::Line, Token::Start]),
	((Token::TMinus, Token::Term), vec![Token::Fact, Token::TermP]),
	((Token::TMinus, Token::TermP), vec![]),
	((Token::TNl, Token::ExprP), vec![]),
	((Token::TNl, Token::Line), vec![Token::TNl,]),
	((Token::TNl, Token::Start), vec![Token::Line, Token::Start]),
	((Token::TNl, Token::TermP), vec![]),
	((Token::TNumber, Token::Expr), vec![Token::Term, Token::ExprP]),
	((Token::TNumber, Token::Fact), vec![Token::TNumber, Token::Push]),
	((Token::TNumber, Token::Line), vec![Token::Expr, Token::Print, Token::TNl]),
	((Token::TNumber, Token::Start), vec![Token::Line, Token::Start]),
	((Token::TNumber, Token::Term), vec![Token::Fact, Token::TermP]),
	((Token::TOp, Token::Expr), vec![Token::Term, Token::ExprP]),
	((Token::TOp, Token::Fact), vec![Token::TOp, Token::Expr, Token::TCp]),
	((Token::TOp, Token::Line), vec![Token::Expr, Token::Print, Token::TNl]),
	((Token::TOp, Token::Start), vec![Token::Line, Token::Start]),
	((Token::TOp, Token::Term), vec![Token::Fact, Token::TermP]),
	((Token::TPlus, Token::ExprP), vec![Token::TPlus, Token::Term, Token::Add, Token::ExprP]),
	((Token::TPlus, Token::TermP), vec![]),
	((Token::TTimes, Token::TermP), vec![Token::TTimes, Token::Fact, Token::Times, Token::TermP])
    ]);

    let mut value_stack: Vec<f64> = Vec::new();
    let mut stack: Vec<Token> = Vec::new();
    stack.push(Token::Start);
    let mut c: char = '\0';
    let mut token = lex(&mut c);
    let mut val = 0.0;
    loop {
        if TRACE {
	    print!("token {:#?}, {} stack", token.0, token.1);
            for v in &stack {
	        print!(" {:#?}", v);
	    }
	    println!();
        }
	match stack.pop() {
	    Some(current_state) => {

		match current_state {
		    Token::Negate => {
			let a = value_stack.pop().unwrap();
			value_stack.push(-a);
		    }
		    Token::Add => {
			let b = value_stack.pop().unwrap();
			let a = value_stack.pop().unwrap();
			value_stack.push(a + b);
		    }
		    Token::Subtract => {
			let b = value_stack.pop().unwrap();
			let a = value_stack.pop().unwrap();
			value_stack.push(a - b);
		    }
		    Token::Times => { 
			let b = value_stack.pop().unwrap();
			let a = value_stack.pop().unwrap();
			value_stack.push(a * b);
		    }
		    Token::Divide => { 
			let b = value_stack.pop().unwrap();
			let a = value_stack.pop().unwrap();
			value_stack.push(a / b);
		    }
		    Token::Push => { 
			value_stack.push(val);
		    }
		    Token::Print => {
			let a = value_stack.pop().unwrap();
			println!("result = {}", a);
		    }
		    Token::TOp | Token::TCp | Token::TNumber | Token::TPlus | Token::TMinus |
		    Token::TTimes | Token::TDivide | Token::TNl | Token::TEnd => {
			if current_state != token.0 {
			    println!("syntax error");
			    return ExitCode::from(1);
			}
			if current_state == Token::TNumber {
			    val = token.1
			}
			token = lex(&mut c);
		    }
		    _ => {
			if !table.contains_key(&(token.0, current_state)) {
			    println!("syntax error");
			    return ExitCode::from(1);
			}
			let _new_bits = &table[&(token.0, current_state)];
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
		}
	    }
	    None => { break; }
	}
    }
    ExitCode::SUCCESS
}
