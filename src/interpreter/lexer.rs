use std::collections::{HashSet, VecDeque};

#[derive(Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(f64),
    Symbol(char),
}
impl Token {
    #[allow(dead_code)]
    pub(crate) fn is_term_symbol(&self) -> bool {
        match self {
            Token::Symbol(s) => s == &'+' || s == &'-' || s == &'*' || s == &'/',
            _ => false,
        }
    }
}

pub struct Lexer {
    pub input: String,
    symbols: HashSet<char>,
    keywords: HashSet<String>,
}

impl Lexer {
    pub fn new() -> Lexer {
        Lexer {
            input: String::new(),
            symbols: HashSet::from(['+', '-', '*', '/', '(', ')', '^', '=', '|']),
            keywords: HashSet::from(
                ["abs", "sqrt", "pow", "pi", "e"].map(|s: &str| -> String { s.to_string() }),
            ),
        }
    }

    pub fn lex(&mut self, arg: &str) -> Result<VecDeque<Token>, super::InterpreterError> {
        self.input = String::from(arg);
        let mut tokens: VecDeque<Token> = VecDeque::new();
        let mut current_token = String::new();

        for c in arg.chars() {
            if c.is_whitespace() {
                continue;
            }

            match self.symbols.get(&c) {
                Some(_) => {
                    if !current_token.is_empty() {
                        tokens.push_back(Token::Identifier(current_token.clone()));
                        current_token = String::new();
                    }
                    tokens.push_back(Token::Symbol(c));
                }
                None => {
                    if self.keywords.contains(&current_token) {
                        if !current_token.is_empty() {
                            tokens.push_back(Token::Identifier(current_token.clone()));
                            current_token = String::new();
                        }
                        current_token.push(c);
                    } else if c.is_numeric() {
                        if !current_token.is_empty() {
                            tokens.push_back(Token::Identifier(current_token.clone()));
                            current_token = String::new();
                        }
                        tokens.push_back(Token::Number(c.to_digit(10).unwrap() as f64));
                    } else {
                        current_token.push(c);
                    }
                }
            }
        }

        // add last token if it exists
        // this is to handle where the last token is an identifier because this is an edge case
        if current_token.len() > 0 {
            tokens.push_back(Token::Identifier(current_token.clone()));
        }
        //println!("{:#?}", tokens);
        Ok(tokens)
    }
}
