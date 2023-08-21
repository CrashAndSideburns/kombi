use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};
use std::process::exit;

use pest::error::{Error, ErrorVariant};
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
            .unwrap_or_else(|e| {
                eprintln!("{}", e);
                exit(1);
            })
            .next()
            .unwrap();
        LambdaTerm::from_pair(parsed, HashMap::new())
    }

    fn from_pair(pair: Pair<Rule>, mut ctx: HashMap<String, u64>) -> Self {
        match pair.as_rule() {
            Rule::variable => {
                let idx = *ctx.get(pair.as_str()).unwrap_or_else(|| {
                    let e = Error::new_from_span(
                        ErrorVariant::<()>::CustomError {
                            message: format!("variable {} is not bound", pair.as_str()),
                        },
                        pair.as_span(),
                    );
                    eprintln!("{}", e);
                    exit(1);
                });
                LambdaTerm::Variable(Variable::new(idx))
            }
            Rule::abstraction => {
                let mut pairs = pair.into_inner();
                let variable = pairs.next().unwrap();
                let body = pairs.next().unwrap();

                // Update the context.
                for v in ctx.values_mut() {
                    *v += 1;
                }
                ctx.insert(variable.as_str().to_string(), 0);

                // Parse the body in the updated context.
                LambdaTerm::Abstraction(Abstraction::new(LambdaTerm::from_pair(body, ctx)))
            }
            Rule::application => {
                let mut pairs = pair.into_inner();
                let function = LambdaTerm::from_pair(pairs.next().unwrap(), ctx.clone());
                let argument = LambdaTerm::from_pair(pairs.next().unwrap(), ctx.clone());

                LambdaTerm::Application(pairs.fold(Application::new(function, argument), |a, p| {
                    Application::new(
                        LambdaTerm::Application(a),
                        LambdaTerm::from_pair(p, ctx.clone()),
                    )
                }))
            }
            _ => unreachable!(),
        }
    }
}

impl Display for LambdaTerm {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            LambdaTerm::Variable(v) => {
                write!(f, "{}", v.idx)
            }
            LambdaTerm::Abstraction(a) => {
                write!(f, "λ {}", a.body)
            }
            LambdaTerm::Application(a) => {
                if let LambdaTerm::Abstraction(_) = *a.function {
                    write!(f, "({}) {}", a.function, a.argument)
                } else if let LambdaTerm::Application(_) = *a.argument {
                    write!(f, "{} ({})", a.function, a.argument)
                } else {
                    write!(f, "{} {}", a.function, a.argument)
                }
            }
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
