pub mod interpreter;
pub mod util;

use crate::interpreter::Interpreter;

fn main() {
    let mut interpreter = Interpreter::new();

    // TODO: error handling
    match interpreter.lex("-2x^2 + -2x^2 + -2x^2") {
        Ok(tokens) => {
            //print!("{:#?}", tokens);

            match interpreter.parse(tokens) {
                Ok(ast_head) => {
                    //print!("{:#?}", ast_head);

                    match interpreter.interpret(ast_head) {
                        Ok(output) => println!("{}", output),
                        Err(e) => {
                            eprintln!("Interpretation failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Parsing failed: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Lexing failed: {}", e);
        }
    }
}
