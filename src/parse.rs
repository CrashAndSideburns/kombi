use std::collections::{hash_map::DefaultHasher, HashMap, VecDeque};
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq)]
/// A token in the formal grammar of our lambda calculus.
enum Token {
    OpenParenthesis,
    CloseParenthesis,
    Lambda,
    Dot,
    // TODO: Allow for a broader class of identifiers. Something like [a-zA-Z_]\w* would suffice.
    Id(char),
}

impl Token {
    /// Create a token from some character if it is contained in our grammar.
    fn new(char: char) -> Option<Self> {
        match char {
            '(' => Some(Token::OpenParenthesis),
            ')' => Some(Token::CloseParenthesis),
            'λ' | '\\' => Some(Token::Lambda),
            '.' => Some(Token::Dot),
            'a'..='z' | 'A'..='Z' => Some(Token::Id(char)),
            _ => None,
        }
    }
}

/// A string of tokens obtained by tokenising an expression.
struct TokenStream(VecDeque<Token>);

impl TokenStream {
    /// Convert a string into a TokenStream. Characters not in our grammar are ignored.
    fn new(string: &str) -> Self {
        TokenStream(string.chars().filter_map(Token::new).collect())
    }
}

#[derive(Debug, Clone, Copy)]
/// A representation of a variable in the lambda calculus.
pub struct Variable {
    /// If this variable is bound, idx will identify this variable by a de Bruijn index. If the
    /// variable is free, idx may identify this variable using any consistent naming context.
    idx: u64,
}

impl Variable {
    fn parse(name: char, ctx: &HashMap<char, u64>) -> Self {
        let idx = ctx.get(&name).copied().unwrap_or_else(|| {
            let mut hasher = DefaultHasher::new();
            name.hash(&mut hasher);
            hasher.finish()
        });
        Variable { idx }
    }
}

#[derive(Debug, Clone)]
/// A representation of abstraction (a lambda function) in the lambda calculus.
pub struct Abstraction {
    body: Box<LambdaTerm>,
}

impl Abstraction {
    /// Parse an abstraction from the supplied TokenStream given a context. The supplied context
    /// defines the bound variables, as well as the de Bruijn indices of bound variables at the
    /// level of nesting of the body of this Abstraction.
    fn parse(token_stream: &mut TokenStream, ctx: &mut HashMap<char, u64>) -> Self {
        // (λ

        // We have already consumed an OpenParenthesis and a Lambda to verify that this term is an
        // Abstraction, so verifying again is unnecessary.
        token_stream.0.pop_front();
        token_stream.0.pop_front();

        match token_stream.0.pop_front() {
            Some(Token::Id(char)) => {
                // Update the context. At the current level of nesting, the de Bruijn index of char
                // is 0.
                ctx.insert(char, 0);
            }
            _ => {
                panic!("Invalid expression.")
            }
        };
        // (λ[a-zA-Z]

        match token_stream.0.pop_front() {
            Some(Token::Dot) => {}
            _ => {
                panic!("Invalid expression.")
            }
        }
        // (λ[a-zA-Z].

        let body = Box::new(LambdaTerm::parse(token_stream, ctx));
        // (λ[a-zA-Z].{LambdaTerm}

        match token_stream.0.pop_front() {
            Some(Token::CloseParenthesis) => {}
            _ => {
                panic!("Invalid expression.")
            }
        }
        // (λ[a-z].{LambdaTerm})

        Abstraction { body }
    }
}

#[derive(Debug, Clone)]
/// A representation of function application in the lambda calculus.
pub struct Application {
    pub function: Box<LambdaTerm>,
    pub argument: Box<LambdaTerm>,
}

impl Application {
    /// Parse an Application from the supplied TokenStream given a context. The supplied context
    /// defines the bound variables, as well as the de Bruijn indices of bound variables at the
    /// level of nesting of the body of this Application.
    fn parse(token_stream: &mut TokenStream, ctx: &mut HashMap<char, u64>) -> Self {
        // (

        token_stream.0.pop_front();

        let function = Box::new(LambdaTerm::parse(token_stream, ctx));
        // ({LambdaTerm}

        let argument = Box::new(LambdaTerm::parse(token_stream, ctx));
        // ({LambdaTerm}{LambdaTerm}

        match token_stream.0.pop_front() {
            Some(Token::CloseParenthesis) => {}
            _ => panic!("Invalid expression."),
        }
        // ({LambdaTerm}{LambdaTerm})

        Application { function, argument }
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
    /// Create a new expression from the given string, according to our grammar.
    pub fn new(string: &str) -> Self {
        LambdaTerm::parse(&mut TokenStream::new(string), &mut HashMap::new())
    }

    /// Parse a LambdaTerm from the supplied TokenStream given a context. The supplied context
    /// defines the bound variables, as well as the de Bruijn indices of bound variables at the
    /// level of nesting of the body of this LambdaTerm.
    fn parse(token_stream: &mut TokenStream, ctx: &mut HashMap<char, u64>) -> Self {
        match token_stream.0.pop_front() {
            Some(Token::OpenParenthesis) => {
                match token_stream.0.pop_front() {
                    Some(Token::Lambda) => {
                        // (λ
                        // parse abstraction
                        token_stream.0.push_front(Token::Lambda);
                        token_stream.0.push_front(Token::OpenParenthesis);
                        LambdaTerm::Abstraction(Abstraction::parse(
                            token_stream,
                            &mut ctx.iter().map(|(k, v)| (*k, v + 1)).collect(),
                        ))
                    }
                    Some(other) => {
                        // ([^λ]
                        // parse application
                        token_stream.0.push_front(other);
                        token_stream.0.push_front(Token::OpenParenthesis);
                        LambdaTerm::Application(Application::parse(token_stream, ctx))
                    }
                    None => {
                        // (
                        // incomplete expression
                        panic!("Invalid expression!")
                    }
                }
            }
            Some(Token::Id(char)) => LambdaTerm::Variable(Variable::parse(char, ctx)),
            _ => panic!("Invalid expression."),
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
