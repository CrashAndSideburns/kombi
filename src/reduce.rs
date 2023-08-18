use crate::parse::*;

impl LambdaTerm {
    /// Replace every instance of the variable with de Bruijn index idx in the current scope with a
    /// LambdaTerm.
    fn replace_idx(&self, new: Self, idx: u64) -> Self {
        match self {
            LambdaTerm::Variable(v) => {
                if v.idx == idx {
                    new
                } else {
                    self.clone()
                }
            }
            LambdaTerm::Abstraction(a) => {
                LambdaTerm::Abstraction(Abstraction::new(a.body.replace_idx(new, idx + 1)))
            }
            LambdaTerm::Application(a) => LambdaTerm::Application(Application::new(
                a.function.replace_idx(new.clone(), idx),
                a.argument.replace_idx(new, idx),
            )),
        }
    }

    /// Apply β-reduction to a given expression in the lambda calculus.
    pub fn beta_reduce(&self) -> Self {
        match self {
            LambdaTerm::Application(a) => {
                // NOTE: It is probably worth noting that this is essentially where the decision to
                // evaluate lazily is being made. Observe that the (prospective) function is
                // β-reduced, but that the argument is substituted directly in, rather than being
                // β-reduced itself prior to substitution. β-reduction is then applied
                // post-substitution.
                match a.function.beta_reduce() {
                    LambdaTerm::Variable(_) => {
                        // HACK: I'm not actually totally sure whether or not this code is
                        // reachable. I have an inkling that this branch is only accessible for
                        // terms which contain free variables.
                        todo!();
                    }
                    LambdaTerm::Application(_) => {
                        // HACK: I'm also not sure whether or not this code is reachable.
                        todo!();
                    }
                    LambdaTerm::Abstraction(b) => {
                        b.body.replace_idx(*a.argument.clone(), 0).beta_reduce()
                    }
                }
            }
            _ => self.clone(),
        }
    }
}
