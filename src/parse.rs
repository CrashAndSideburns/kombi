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

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    BaseType(String),
    FunctionType(Box<Type>, Box<Type>),
}

impl Type {
    fn from_pair(pair: Pair<Rule>) -> Self {
        match pair.as_rule() {
            Rule::base_type => Type::BaseType(pair.as_str().to_string()),
            Rule::function_type => {
                let mut pairs = pair.into_inner();
                let argument_type = Box::new(Self::from_pair(pairs.next().unwrap()));
                let return_type = Box::new(Self::from_pair(pairs.next().unwrap()));
                Type::FunctionType(argument_type, return_type)
            }
            _ => unreachable!(),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Type::BaseType(name) => name.fmt(f),
            Type::FunctionType(argument_type, return_type) => match **argument_type {
                Type::BaseType(_) => write!(f, "{argument_type}→{return_type}"),
                Type::FunctionType(..) => write!(f, "({argument_type})→{return_type}"),
            },
        }
    }
}

#[derive(Debug, Clone)]
/// A representation of an arbitrary expression in the lambda calculus.
pub enum LambdaTerm {
    Variable {
        idx: u64,
    },
    Abstraction {
        argument_type: Type,
        body: Box<LambdaTerm>,
    },
    Application {
        function: Box<LambdaTerm>,
        argument: Box<LambdaTerm>,
    },
}

impl LambdaTerm {
    /// Create a new `LambdaTerm` from the given string, according to our grammar.
    pub fn from_str(string: &str) -> Self {
        let parsed = KombiParser::parse(Rule::program, string)
            .unwrap_or_else(|e| {
                eprintln!("{e}");
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
                    eprintln!("{e}");
                    exit(1);
                });
                LambdaTerm::Variable { idx }
            }
            Rule::abstraction => {
                let mut pairs = pair.into_inner();
                let variable = pairs.next().unwrap();
                let argument_type = Type::from_pair(pairs.next().unwrap());
                let body = pairs.next().unwrap();

                // Update the context.
                for v in ctx.values_mut() {
                    *v += 1;
                }
                ctx.insert(variable.as_str().to_string(), 0);

                // Parse the body in the updated context.
                LambdaTerm::Abstraction {
                    argument_type,
                    body: Box::new(LambdaTerm::from_pair(body, ctx)),
                }
            }
            Rule::application => {
                let mut pairs = pair.into_inner();
                let function = Box::new(LambdaTerm::from_pair(pairs.next().unwrap(), ctx.clone()));
                let argument = Box::new(LambdaTerm::from_pair(pairs.next().unwrap(), ctx.clone()));

                pairs.fold(LambdaTerm::Application { function, argument }, |a, p| {
                    LambdaTerm::Application {
                        function: Box::new(a),
                        argument: Box::new(LambdaTerm::from_pair(p, ctx.clone())),
                    }
                })
            }
            _ => unreachable!(),
        }
    }
}

impl Display for LambdaTerm {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            LambdaTerm::Variable { idx } => {
                write!(f, "{idx}")
            }
            LambdaTerm::Abstraction {
                argument_type,
                body,
            } => {
                write!(f, "λ:{argument_type} {body}")
            }
            LambdaTerm::Application { function, argument } => {
                if let LambdaTerm::Abstraction { .. } = **function {
                    write!(f, "({function}) {argument}")
                } else if let LambdaTerm::Application { .. } = **argument {
                    write!(f, "{function} ({argument})")
                } else {
                    write!(f, "{function} {argument}")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
