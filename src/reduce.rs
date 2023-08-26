use crate::parse::LambdaTerm;

impl LambdaTerm {
    /// Replace every instance of the variable with de Bruijn index `replacement_idx` in the
    /// current scope with a `LambdaTerm`.
    fn replace_idx(&self, new: Self, replacement_idx: u64) -> Self {
        match self {
            LambdaTerm::Variable { idx } => {
                if idx == &replacement_idx {
                    new
                } else {
                    self.clone()
                }
            }
            LambdaTerm::Abstraction { body } => LambdaTerm::Abstraction {
                body: Box::new(body.replace_idx(new, replacement_idx + 1)),
            },
            LambdaTerm::Application { function, argument } => LambdaTerm::Application {
                function: Box::new(function.replace_idx(new.clone(), replacement_idx)),
                argument: Box::new(argument.replace_idx(new, replacement_idx)),
            },
        }
    }

    /// Apply β-reduction to a given expression in the lambda calculus.
    pub fn beta_reduce(&self) -> Self {
        match self {
            LambdaTerm::Application { function, argument } => {
                // NOTE: It is probably worth noting that this is essentially where the decision to
                // evaluate lazily is being made. Observe that the (prospective) function is
                // β-reduced, but that the argument is substituted directly in, rather than being
                // β-reduced itself prior to substitution. β-reduction is then applied
                // post-substitution.
                match function.beta_reduce() {
                    LambdaTerm::Abstraction { body } => {
                        body.replace_idx(*argument.clone(), 0).beta_reduce()
                    }
                    _ => {
                        // NOTE: This would only be reachable when β-reducing terms which contain
                        // free variables, which are not allowed in our grammar.
                        unreachable!()
                    }
                }
            }
            _ => self.clone(),
        }
    }
}
