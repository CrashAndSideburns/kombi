use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::parse::{LambdaTerm, Type};

// NOTE: For now, the only error which the type checker may encounter is an attempt to apply a
// function which does not take a term of type T as an argument to a term of type T. This is left
// as an enum in case future expansion of the type system leads to more possible errors.
#[derive(Debug)]
pub enum TypeError {
    InvalidApplication {
        function: LambdaTerm,
        function_type: Type,
        argument: LambdaTerm,
        argument_type: Type,
    },
}

impl Display for TypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidApplication {
                function,
                function_type,
                argument,
                argument_type,
            } => {
                write!(f, "attempted to apply term ({function}):{function_type} to term ({argument}):{argument_type}")
            }
        }
    }
}

impl Error for TypeError {}

impl LambdaTerm {
    /// Return the `Type` of the `LambaTerm` if it is well-typed, or an appropriate `TypeError` if
    /// it is not.
    pub fn get_type(&self) -> Result<Type, TypeError> {
        self.get_type_in_context(Vec::new())
    }

    fn get_type_in_context(&self, mut ctx: Vec<Type>) -> Result<Type, TypeError> {
        match self {
            LambdaTerm::Variable { idx } => Ok(ctx.swap_remove(ctx.len() - (idx + 1) as usize)),
            LambdaTerm::Abstraction {
                argument_type,
                body,
            } => {
                ctx.push(argument_type.clone());
                let return_type = body.get_type_in_context(ctx)?;

                Ok(Type::FunctionType(
                    Box::new(argument_type.clone()),
                    Box::new(return_type),
                ))
            }
            LambdaTerm::Application { function, argument } => {
                let function_type = function.get_type_in_context(ctx.clone())?;
                let argument_type = argument.get_type_in_context(ctx)?;

                if let Type::FunctionType(function_argument_type, return_type) =
                    function_type.clone()
                {
                    if *function_argument_type == argument_type {
                        Ok(*return_type)
                    } else {
                        Err(TypeError::InvalidApplication {
                            function: *function.clone(),
                            function_type,
                            argument: *argument.clone(),
                            argument_type,
                        })
                    }
                } else {
                    Err(TypeError::InvalidApplication {
                        function: *function.clone(),
                        function_type,
                        argument: *argument.clone(),
                        argument_type,
                    })
                }
            }
        }
    }
}
