use std::collections::VecDeque;

#[derive(Debug, PartialEq)]
/// A token in the formal grammar of our lambda calculus.
enum Token {
    OpenParenthesis,
    CloseParenthesis,
    Lambda,
    Dot,
    Id(char),
}

impl Token {
    /// Create a token from some character.
    /// If the character passed to this function is not one that is in our grammar, return None.
    fn new(char: char) -> Option<Self> {
        match char {
            '(' => Some(Token::OpenParenthesis),
            ')' => Some(Token::CloseParenthesis),
            'λ' | '\\' => Some(Token::Lambda),
            '.' => Some(Token::Dot),
            'a'..='z' => Some(Token::Id(char)),
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

#[derive(Debug)]
/// A representation of a variable in the lambda calculus.
pub struct Variable {
    name: String,
}

impl Variable {
    /// Create a Variable with the given name.
    fn new(name: String) -> Self {
        Variable { name }
    }
}

#[derive(Debug)]
/// A representation of abstraction (a lambda function) in the lambda calculus.
pub struct Abstraction {
    variable: Variable,
    body: Box<LambdaTerm>,
}

impl Abstraction {
    /// Parse an abstraction from the supplied TokenStream.
    /// This will readily panic if the tokens in the TokenStream do not define a valid Abstraction.
    fn parse(token_stream: &mut TokenStream) -> Self {
        // (λ

        token_stream.0.pop_front();
        token_stream.0.pop_front();

        let variable = match token_stream.0.pop_front() {
            Some(Token::Id(char)) => Variable::new(char.to_string()),
            _ => {
                panic!("Invalid expression.")
            }
        };
        // (λ[a-z]

        match token_stream.0.pop_front() {
            Some(Token::Dot) => {}
            _ => {
                panic!("Invalid expression.")
            }
        }
        // (λ[a-z].

        let body = Box::new(LambdaTerm::parse(token_stream));
        // (λ[a-z].{LambdaTerm}

        match token_stream.0.pop_front() {
            Some(Token::CloseParenthesis) => {}
            _ => {
                panic!("Invalid expression.")
            }
        }
        // (λ[a-z].{LambdaTerm})

        Abstraction { variable, body }
    }
}

#[derive(Debug)]
/// A representation of function application in the lambda calculus.
pub struct Application {
    function: Box<LambdaTerm>,
    argument: Box<LambdaTerm>,
}

impl Application {
    /// Parse an application from the supplied TokenStream.
    /// This will readily panic if the tokens in the TokenStream do not define a valid Application.
    fn parse(token_stream: &mut TokenStream) -> Self {
        // (

        token_stream.0.pop_front();

        let function = Box::new(LambdaTerm::parse(token_stream));
        // ({LambdaTerm}

        let argument = Box::new(LambdaTerm::parse(token_stream));
        // ({LambdaTerm}{LambdaTerm}

        match token_stream.0.pop_front() {
            Some(Token::CloseParenthesis) => {}
            _ => panic!("Invalid expression."),
        }
        // ({LambdaTerm}{LambdaTerm})

        Application { function, argument }
    }
}

#[derive(Debug)]
/// A representation of an arbitrary expression in the lambda calculus.
pub enum LambdaTerm {
    Variable(Variable),
    Abstraction(Abstraction),
    Application(Application),
}

impl LambdaTerm {
    /// Create a new expression from the given string, according to our grammar.
    pub fn new(string: &str) -> Self {
        LambdaTerm::parse(&mut TokenStream::new(string))
    }

    /// Parse an expression from the supplied TokenStream.
    /// This will readily panic if the tokens in the TokenStream do not define a valid LambdaTerm.
    fn parse(token_stream: &mut TokenStream) -> Self {
        match token_stream.0.pop_front() {
            Some(Token::OpenParenthesis) => {
                match token_stream.0.pop_front() {
                    Some(Token::Lambda) => {
                        // (λ
                        // parse abstraction
                        token_stream.0.push_front(Token::Lambda);
                        token_stream.0.push_front(Token::OpenParenthesis);
                        LambdaTerm::Abstraction(Abstraction::parse(token_stream))
                    }
                    Some(other) => {
                        // ([^λ]
                        // parse application
                        token_stream.0.push_front(other);
                        token_stream.0.push_front(Token::OpenParenthesis);
                        LambdaTerm::Application(Application::parse(token_stream))
                    }
                    None => {
                        // (
                        // incomplete expression
                        panic!("Invalid expression!")
                    }
                }
            }
            Some(Token::Id(char)) => LambdaTerm::Variable(Variable::new(char.to_string())),
            _ => panic!("Invalid expression."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenisation() {
        let test_string = "()λ#\\.az?";
        let token_stream = TokenStream::new(test_string);
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
