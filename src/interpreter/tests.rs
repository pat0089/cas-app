use super::*;

fn lex(input: &str) -> Result<VecDeque<lexer::Token>, InterpreterError> {
    let mut interpreter = Interpreter::new();
    match interpreter.lex(input) {
        Ok(tokens) => Ok(tokens),
        Err(e) => Err(InterpreterError::new(format!("Lexing failed: {}", e))),
    }
}

fn parse(tokens: VecDeque<lexer::Token>) -> Result<parser::ASTNode, InterpreterError> {
    let mut interpreter = Interpreter::new();
    match interpreter.parse(tokens) {
        Ok(ast) => Ok(ast),
        Err(e) => Err(InterpreterError::new(format!("Parsing failed: {}", e))),
    }
}

fn interpret(input: &str) -> Result<String, InterpreterError> {
    let interpreter = Interpreter::new();
    match interpreter.interpret(parse(lex(input)?)?) {
        Ok(output) => Ok(output),
        Err(e) => Err(InterpreterError::new(format!(
            "Interpretation failed: {}",
            e
        ))),
    }
}

#[test]
fn full_basic_test() {
    let output = interpret("1 + 2").expect("Interpretation failed");

    assert_eq!(output, "3");
}

#[test]
fn lexer_basic_test() -> Result<(), InterpreterError> {
    let tokens = lex("1 + 2")?;

    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0], lexer::Token::Number(1.0));
    assert_eq!(tokens[1], lexer::Token::Symbol('+'));
    assert_eq!(tokens[2], lexer::Token::Number(2.0));
    Ok(())
}

#[test]
fn parser_basic_test() -> Result<(), InterpreterError> {
    let ast = parse(lex("1 + 2")?)?;

    assert_eq!(
        ast,
        parser::ASTNode::Expression(vec![
            parser::ASTNode::Term(Box::new(parser::ASTNode::Number(1.0)), Vec::new(),),
            parser::ASTNode::Term(Box::new(parser::ASTNode::Number(2.0)), Vec::new(),),
        ])
    );

    Ok(())
}

#[test]
fn empty_input_test() -> Result<(), InterpreterError> {
    let output = interpret("")?;
    assert!(output.is_empty());
    Ok(())
}

#[test]
fn same_order_input_output_test() -> Result<(), InterpreterError> {
    let input = "100x^2 + 200x + 300";

    let output = interpret(input)?;

    assert_eq!(output, input);

    Ok(())
}

#[test]
fn sort_terms_test() -> Result<(), InterpreterError> {
    let expected = "100x^2 + 200x + 300";

    let input = "200x + 100x^2 + 300";
    let input1 = "300 + 200x + 100x^2";
    let input2 = "300 + 100x^2 + 200x";

    let output = interpret(input)?;
    let output1 = interpret(input1)?;
    let output2 = interpret(input2)?;

    assert_eq!(output, expected);
    assert_eq!(output1, expected);
    assert_eq!(output2, expected);

    Ok(())
}

#[test]
fn multiple_variables_test() -> Result<(), InterpreterError> {
    let input = "x^2 + xy + y^2";
    let input1 = "y^2 + xy + x^2";
    let input2 = "y^2 + x^2 + xy";
    let input3 = "xy + y^2 + x^2";

    let output = interpret(input)?;
    let output1 = interpret(input1)?;
    let output2 = interpret(input2)?;
    let output3 = interpret(input3)?;

    assert_eq!(output, input);
    assert_eq!(output1, input);
    assert_eq!(output2, input);
    assert_eq!(output3, input);

    Ok(())
}

#[test]
fn multiple_variables_with_coefficients_test() -> Result<(), InterpreterError> {
    let input = "2x^2 + 2xy + 2y^2";
    let input1 = "2y^2 + 2xy + 2x^2";
    let input2 = "2y^2 + 2x^2 + 2xy";
    let input3 = "2xy + 2y^2 + 2x^2";

    let output = interpret(input)?;
    let output1 = interpret(input1)?;
    let output2 = interpret(input2)?;
    let output3 = interpret(input3)?;

    assert_eq!(output, input);
    assert_eq!(output1, input);
    assert_eq!(output2, input);
    assert_eq!(output3, input);

    Ok(())
}

#[test]
fn multiple_variables_with_coefficients_and_exponents_test() -> Result<(), InterpreterError> {
    let expected = "2x^3y + 2xy^3 + 2xy^2";

    let input = "2xy^3 + 2xy^2 + 2yx^3";
    let input1 = "2yx^3 + 2xy^2 + 2xy^3";
    let input2 = "2yx^3 + 2xy^3 + 2xy^2";
    let input3 = "2xy^2 + 2yx^3 + 2xy^3";

    let output = interpret(input)?;
    let output1 = interpret(input1)?;
    let output2 = interpret(input2)?;
    let output3 = interpret(input3)?;

    assert_eq!(output, expected);
    assert_eq!(output1, expected);
    assert_eq!(output2, expected);
    assert_eq!(output3, expected);

    Ok(())
}

#[test]
fn sort_alphabetically_test() -> Result<(), InterpreterError> {
    let input = "xyz";
    let input1 = "yxz";
    let input2 = "yzx";
    let input3 = "xzy";

    let output = interpret(input)?;
    let output1 = interpret(input1)?;
    let output2 = interpret(input2)?;
    let output3 = interpret(input3)?;

    assert_eq!(output, input);
    assert_eq!(output1, input);
    assert_eq!(output2, input);
    assert_eq!(output3, input);

    Ok(())
}

#[test]
fn combine_like_terms_test() -> Result<(), InterpreterError> {
    let input = "2x^2 + 2x^2 + 2x^2";
    let input1 = "2y^2 + 2y^2 + 2y^2";
    let input2 = "2y^2 + 2y^2 + 2x^2";
    let input3 = "2x^2 + 2x^2 + 2y^2";

    let output = interpret(input)?;
    let output1 = interpret(input1)?;
    let output2 = interpret(input2)?;
    let output3 = interpret(input3)?;

    assert_eq!(output, "6x^2");
    assert_eq!(output1, "6y^2");
    assert_eq!(output2, "2x^2 + 4y^2");
    assert_eq!(output3, "4x^2 + 2y^2");

    Ok(())
}

#[test]
fn negative_coefficients_test() -> Result<(), InterpreterError> {
    let input = "-2x^2 + -2x^2 + -2x^2";

    let output = interpret(input)?;

    assert_eq!(output, "-6x^2");

    Ok(())
}

#[test]
fn negative_exponents_test() -> Result<(), InterpreterError> {
    let input = "x^-2 + x^-2 + x^-2";

    let output = interpret(input)?;

    assert_eq!(output, "3x^-2");

    Ok(())
}

#[test]
fn zero_test() -> Result<(), InterpreterError> {
    let input = "0";
    let input1 = "0x";
    let input2 = "0x^0";
    let input3 = "0x + 0";
    let input4 = "0x^0 + 0y + 0";

    let output = interpret(input)?;
    let output1 = interpret(input1)?;
    let output2 = interpret(input2)?;
    let output3 = interpret(input3)?;
    let output4 = interpret(input4)?;

    assert_eq!(output, input);
    assert_eq!(output1, input);
    assert_eq!(output2, input);
    assert_eq!(output3, input);
    assert_eq!(output4, input);

    Ok(())
}

#[test]
fn subtractive_terms_test() -> Result<(), InterpreterError> {
    let input = "-1x - 1x - 1x";
    let input2 = "-1x + -1x + -1x";
    let input3 = "-1x + -1x - +1x";

    let input4 = "---1x ---1x ---1x";

    let expected = "-3x";

    let output = interpret(input)?;
    let output2 = interpret(input2)?;
    let output3 = interpret(input3)?;
    let output4 = interpret(input4)?;

    assert_eq!(output, expected);
    assert_eq!(output2, expected);
    assert_eq!(output3, expected);
    assert_eq!(output4, expected);

    Ok(())
}

#[test]
fn error_test() -> Result<(), InterpreterError> {
    let input = "-";
    let mut max_string = f64::MAX.to_string();
    max_string.push('0');
    let input1 = max_string.as_str();

    let output = interpret(input)?;
    let output1 = interpret(input1);

    assert!(output.is_empty());
    assert!(output1.is_err());

    Ok(())
}