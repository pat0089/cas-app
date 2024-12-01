use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet, VecDeque},
};

use crate::util::hashable_float::HashableFloat;

use super::{lexer::Token, InterpreterError};

#[derive(Debug, PartialEq)]
pub enum ASTNode {
    Number(f64),
    Operation(String, Box<ASTNode>, Box<ASTNode>),
    Variable(String, Box<ASTNode>),
    Function(String, Vec<ASTNode>),
    Equation(Box<ASTNode>, Box<ASTNode>),
    Expression(Vec<ASTNode>),
    Term(Box<ASTNode>, Vec<ASTNode>),
}

pub struct ParsedExpression {
    pub terms: HashMap<Vec<(String, HashableFloat)>, f64>,
    variables: HashSet<String>,
}

impl ParsedExpression {
    pub fn new() -> Self {
        ParsedExpression {
            terms: HashMap::new(),
            variables: HashSet::new(),
        }
    }

    pub fn add_term(&mut self, term: Vec<(String, HashableFloat)>, coefficient: f64) {
        if coefficient != 0.0 {
            if term.len() > 0 {
                self.variables.insert(term[0].0.clone());
                self.terms
                    .entry(term)
                    .and_modify(|e| *e += coefficient)
                    .or_insert(coefficient);
            } else {
                self.terms
                    .entry(Vec::new())
                    .and_modify(|e| *e += coefficient)
                    .or_insert(coefficient);
            }
        }
    }

    pub fn get_term(&self, term: Vec<(String, HashableFloat)>) -> Option<f64> {
        self.terms.get(&term).cloned()
    }

    pub fn get_sorted_term_sigs(&self) -> Vec<&Vec<(String, HashableFloat)>> {
        let mut keys = self
            .terms
            .keys()
            .collect::<Vec<&Vec<(String, HashableFloat)>>>();
        keys.sort_by(|a, b| {
            // Handle empty vectors: constants move to the right
            if a.is_empty() || b.is_empty() {
                return b.len().cmp(&a.len());
            }

            // Iterate over each index in both vectors for comparison
            for (elem_a, elem_b) in a.iter().zip(b.iter()) {
                // Sort by variable names alphabetically
                match elem_a.0.partial_cmp(&elem_b.0).unwrap_or(Ordering::Equal) {
                    Ordering::Equal => {}
                    non_equal => return non_equal,
                }

                // Sort by exponent values descending
                match elem_b
                    .1
                     .0
                    .partial_cmp(&elem_a.1 .0)
                    .unwrap_or(Ordering::Equal)
                {
                    Ordering::Equal => {}
                    non_equal => return non_equal,
                }
            }

            // If all elements are equal so far, fall back to comparing lengths
            a.len().cmp(&b.len())
        });
        keys
    }

    pub fn get_variables(&self) -> HashSet<String> {
        self.variables.clone()
    }
}

pub struct Parser {}

impl Parser {
    pub fn parse(&mut self, mut tokens: VecDeque<Token>) -> Result<ASTNode, InterpreterError> {
        self.parse_expression(&mut tokens)
    }

    pub fn parse_expression(
        &mut self,
        tokens: &mut VecDeque<Token>,
    ) -> Result<ASTNode, InterpreterError> {
        let mut terms = Vec::new();
        while tokens.len() > 0 {
            let term = self.parse_term(tokens)?;
            //println!("{:#?}", term);
            terms.push(term);
        }

        println!("{:#?}", terms);

        Ok(ASTNode::Expression(terms))
    }

    fn parse_term(&self, mut tokens: &mut VecDeque<Token>) -> Result<ASTNode, InterpreterError> {
        let before_constant_length = tokens.len();
        let coefficient = parse_constant(&mut tokens)?;
        let after_constant_length = tokens.len();
        let variables = parse_optional_variables(&mut tokens)?;

        //println!("{:#?}, {} - {}", coefficient, before_constant_length, after_constant_length);

        //pop that pesky term operator for now, if it's addition
        if let Some(Token::Symbol('+')) = tokens.front() {
            tokens.pop_front();
        }

        let zero = Box::new(ASTNode::Number(0.0));
        //check for if this is a non-constant term that has a constant of 0, (implicit coefficient of 1)
        //  -> but how do we differentiate this from a zero constant?
        //  Easy: length of token list to determine whether we've read in any numbers
        if variables.len() > 0 && coefficient == zero {
            if before_constant_length == after_constant_length {
                return Ok(ASTNode::Term(Box::new(ASTNode::Number(1.0)), variables));
            } else {
                return Ok(ASTNode::Term(Box::new(ASTNode::Number(0.0)), variables));
            }
        }
        Ok(ASTNode::Term(coefficient, variables))
    }

    pub(crate) fn new() -> Self {
        Parser {}
    }
}

/// parse_optional_exponent
///
/// This function takes a stream of tokens and tries to parse an optional exponent.
/// If an exponent is found, it is parsed and returned as a boxed ASTNode.
/// If an exponent is not found, a boxed ASTNode representing the number 1 is returned.
fn parse_optional_exponent(
    mut tokens: &mut VecDeque<Token>,
) -> Result<Box<ASTNode>, InterpreterError> {
    match tokens.front() {
        Some(Token::Symbol('^')) => {
            tokens.pop_front();
            Ok(parse_constant(&mut tokens)?)
        }
        _ => Ok(Box::new(ASTNode::Number(1.0))),
    }
}

/// parse_optional_variables
///
/// This function takes a stream of tokens and tries to parse a variable.
/// If a variable is found, it is added to a vector, and then any optional exponents are parsed and added to the vector.
/// If a variable is not found, an empty vector is returned.
///
fn parse_optional_variables(
    tokens: &mut VecDeque<Token>,
) -> Result<Vec<ASTNode>, InterpreterError> {
    let option = match tokens.front() {
        Some(Token::Identifier(_)) => true,
        _ => false,
    };

    if option {
        match tokens.pop_front() {
            Some(Token::Identifier(s)) => {
                let var_string = s.clone();
                let mut variables = Vec::new();
                //keep reading variables and optional exponents until we hit something else (identifier has a length greater than 1)
                if var_string.len() > 1 {
                    for c in var_string.chars().take(var_string.len() - 1) {
                        variables.push(ASTNode::Variable(
                            c.to_string(),
                            Box::new(ASTNode::Number(1.0)),
                        ));
                    }
                    let last = var_string.chars().last().unwrap();
                    let optional_exponent = match *parse_optional_exponent(tokens)? {
                        ASTNode::Number(n) => Box::new(ASTNode::Number(n)) as Box<ASTNode>,
                        _ => Box::new(ASTNode::Number(1.0)),
                    };
                    variables.push(ASTNode::Variable(last.to_string(), optional_exponent));
                } else {
                    variables.push(ASTNode::Variable(
                        s.clone(),
                        parse_optional_exponent(tokens)?,
                    ));
                }
                return Ok(variables);
            }
            _ => return Ok(Vec::new()),
        }
    }

    Ok(Vec::new())
}

fn parse_constant(mut tokens: &mut VecDeque<Token>) -> Result<Box<ASTNode>, InterpreterError> {
    let sign = get_sign(&mut tokens);
    let mut accumulator = 0.0;
    loop {
        if let Some(Token::Number(n)) = tokens.front() {
            if accumulator >= f64::MAX / 10.0 {
                return Err(InterpreterError::unsupported_number(accumulator, *n));
            }
            accumulator *= 10.0;
            accumulator += n;
            tokens.pop_front();
        } else {
            break;
        }
    }
    return Ok(Box::new(ASTNode::Number(
        accumulator * if sign { 1.0 } else { -1.0 },
    )));
}

fn get_sign(tokens: &mut VecDeque<Token>) -> bool {
    let mut sign = true;
    loop { 
        let front = tokens.front();
        match front {
            Some(Token::Symbol('-')) => {
                sign = !sign;
                tokens.pop_front();
                continue;
            }
            Some(Token::Symbol('+')) => {
                tokens.pop_front();
                continue;
            }
            _ => return sign,
        }
    }
    //return sign;
}
