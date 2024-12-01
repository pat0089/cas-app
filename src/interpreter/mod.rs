use std::{cmp::Ordering, collections::VecDeque};

use parser::ParsedExpression;

use crate::util::hashable_float::HashableFloat;

pub mod lexer;
pub mod parser;

pub struct Interpreter {
    pub parser: parser::Parser,
    pub lexer: lexer::Lexer,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            parser: parser::Parser::new(),
            lexer: lexer::Lexer::new(),
        }
    }

    pub fn lex(&mut self, arg: &str) -> Result<VecDeque<lexer::Token>, InterpreterError> {
        self.lexer.lex(arg)
    }

    pub fn parse(
        &mut self,
        tokens: VecDeque<lexer::Token>,
    ) -> Result<parser::ASTNode, InterpreterError> {
        self.parser.parse(tokens)
    }

    pub(crate) fn interpret(&self, ast_head: parser::ASTNode) -> Result<String, InterpreterError> {
        let mut output = String::new();
        match ast_head {
            parser::ASTNode::Number(n) => {
                output.push_str(n.to_string().as_str());
                Ok(output)
            }
            parser::ASTNode::Expression(terms) => Ok(self.solve(terms)?),
            _ => Err(InterpreterError::new(
                "Invalid interpretation input".to_string(),
            )),
        }
    }

    /// Combine like terms
    /// example: 2x^2 + 2x^2 -> 4x^2
    ///
    /// adds up the coefficients for each type of term, and adds up constants
    ///
    fn combine_like_terms(
        &self,
        terms: &mut Vec<parser::ASTNode>,
    ) -> Result<ParsedExpression, InterpreterError> {
        //Expression is a mapping of unique term signatures to their coefficients
        let mut accumulator = ParsedExpression::new();

        for term in terms.iter() {
            match term {
                parser::ASTNode::Term(coefficient, variables) => {
                    let coefficient = match **coefficient {
                        parser::ASTNode::Number(n) => n,
                        _ => 0.0,
                    };

                    // combine constants
                    if variables.len() == 0 {
                        accumulator.add_term(Vec::new(), coefficient);
                    } else {
                        // combine variables
                        let mut term_identifier: Vec<(String, HashableFloat)> = Vec::new();
                        for variable in variables.iter() {
                            let var_identifier = match variable {
                                parser::ASTNode::Variable(name, exponent) => {
                                    let e = match **exponent {
                                        parser::ASTNode::Number(n) => n,
                                        _ => 0.0,
                                    };
                                    (name.clone(), HashableFloat::new(e))
                                }
                                _ => (String::new(), HashableFloat::new(1.0)),
                            };
                            //TODO: add handling for variables written out as multiples of the same variable i.e. 'xxx' => x^3
                            term_identifier.push(var_identifier);
                        }

                        // sort by variable, then exponent
                        term_identifier.sort_by(|a, b| {
                            a.0.cmp(&b.0).then_with(|| {
                                a.1 .0.partial_cmp(&b.1 .0).unwrap_or(Ordering::Equal)
                            })
                        });

                        accumulator.add_term(term_identifier, coefficient);
                    }
                }
                _ => (),
            }
        }

        Ok(accumulator)
    }

    fn print_out_expression(&self, expression: ParsedExpression) -> String {
        let mut output_string = String::new();

        // sort by first exponent
        let keys = expression.get_sorted_term_sigs();

        //zero cases
        let mut zero_flag = false;
        for signature in keys.clone().iter() {
            if expression.get_term(signature.to_vec()).unwrap() != 0.0 {
                zero_flag = false;
                break;
            } else {
                zero_flag = true;
            }
        }
        if keys.len() == 0 || zero_flag {
            return "0".to_string();
        }

        //then, access the terms in order and output to a string
        for (i, signature) in keys.iter().enumerate() {
            let coefficient = expression.get_term(signature.to_vec()).unwrap();
            if HashableFloat::new(coefficient) != HashableFloat::new(1.0) || signature.len() == 0 {
                output_string.push_str(&format!("{}", coefficient));
            }
            for (variable, exponent) in signature.iter() {
                if *exponent != HashableFloat::new(1.0) {
                    output_string.push_str(&format!("{}^{}", variable, exponent));
                } else {
                    output_string.push_str(variable);
                }
            }
            if i < keys.len() - 1 {
                output_string.push_str(" + ");
            }
        }

        output_string
    }

    fn solve(&self, mut terms: Vec<parser::ASTNode>) -> Result<String, InterpreterError> {
        if terms.len() == 0 {
            return Ok("".to_string());
        }
        let expression = self.combine_like_terms(&mut terms)?;
        Ok(self.print_out_expression(expression))
    }
}

#[derive(Debug)]
pub struct InterpreterError {
    message: String,
}

impl InterpreterError {
    fn new(message: String) -> InterpreterError {
        InterpreterError { message }
    }

    fn unsupported_number(accumulator: f64, n: f64) -> InterpreterError {
        InterpreterError::new(format!("Unsupported number: {}{}", accumulator, n))
    }
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InterpreterError: {}", self.message)
    }
}

impl std::error::Error for InterpreterError {}

#[cfg(test)]
pub mod tests;
