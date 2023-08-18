use std::collections::HashMap;

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "kombi.pest"]
pub struct KombiParser;

#[derive(Debug, Clone, Copy, PartialEq)]
/// A representation of a variable in the lambda calculus.
pub struct Variable {
    /// If this variable is bound, idx will identify this variable by a de Bruijn index. If the
    /// variable is free, idx may identify this variable using any consistent naming context.
    pub idx: u64,
}

impl Variable {
    /// Create a new variable from a raw de Bruijn index.
    pub fn new(idx: u64) -> Self {
        Self { idx }
    }
}

#[derive(Debug, Clone)]
/// A representation of abstraction (a lambda function) in the lambda calculus.
pub struct Abstraction {
    pub body: Box<LambdaTerm>,
}

impl Abstraction {
    /// Create a new Abstraction from a raw body. If doing this, be very careful to make sure that
    /// your de Bruijn indices are correct.
    pub fn new(body: LambdaTerm) -> Self {
        Self {
            body: Box::new(body),
        }
    }
}

#[derive(Debug, Clone)]
/// A representation of function application in the lambda calculus.
pub struct Application {
    pub function: Box<LambdaTerm>,
    pub argument: Box<LambdaTerm>,
}

impl Application {
    /// Create a new Application from a raw function and argument.
    pub fn new(function: LambdaTerm, argument: LambdaTerm) -> Self {
        Self {
            function: Box::new(function),
            argument: Box::new(argument),
        }
    }
}

#[derive(Debug, Clone)]
/// A representation of an arbitrary expression in the lambda calculus.
pub enum LambdaTerm {
    Variable(Variable),
    Abstraction(Abstraction),
    Application(Application),
}

impl LambdaTerm {
    /// Create a new LambdaTerm from the given string, according to our grammar.
    pub fn from_str(string: &str) -> Self {
        let parsed = KombiParser::parse(Rule::program, string)
            .expect("Failed to parse.")
            .next()
            .unwrap();
        LambdaTerm::from_pair(parsed, &mut HashMap::new())
    }

    fn from_pair(pair: Pair<Rule>, ctx: &mut HashMap<String, u64>) -> Self {
        match pair.as_rule() {
            Rule::variable => {
                let idx = *ctx.get(pair.as_str()).expect("Free variable found.");
                LambdaTerm::Variable(Variable::new(idx))
            }
            Rule::abstraction => {
                let mut pairs = pair.into_inner();
                let variable = pairs.next().unwrap();
                let body = pairs.next().unwrap();

                // Update the context.
                ctx.values_mut().map(|v| *v += 1);
                ctx.insert(variable.as_str().to_string(), 0);

                // Parse the body in the updated context.
                LambdaTerm::Abstraction(Abstraction::new(LambdaTerm::from_pair(body, ctx)))
            }
            Rule::application => {
                let mut pairs = pair.into_inner();
                let function = LambdaTerm::from_pair(pairs.next().unwrap(), ctx);
                let argument = LambdaTerm::from_pair(pairs.next().unwrap(), ctx);

                LambdaTerm::Application(Application::new(function, argument))
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenisation() {
        let token_stream = TokenStream::new("()λ#\\.az?");
        assert_eq!(
            token_stream.0,
            vec![
                Token::OpenParenthesis,
                Token::CloseParenthesis,
                Token::Lambda,
                Token::Lambda,
                Token::Dot,
                Token::Id('a'),
                Token::Id('z')
            ]
        );
    }
}
