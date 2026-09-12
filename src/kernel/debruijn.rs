use super::LeanExpr;

pub(super) fn lookup_bvar_type(context: &[LeanExpr], index: usize) -> Result<LeanExpr, String> {
    let stored = context
        .get(index)
        .ok_or_else(|| format!("Bound variable #{index} is out of scope"))?;

    let distance = index
        .checked_add(1)
        .ok_or_else(|| "Bound variable lookup distance overflow".to_string())?;

    stored.lift(distance, 0)
}

impl LeanExpr {
    pub fn lift(&self, amount: usize, cutoff: usize) -> Result<LeanExpr, String> {
        match self {
            LeanExpr::Sort(level) => Ok(LeanExpr::Sort(level.clone())),

            LeanExpr::BVar(index) => {
                if *index < cutoff {
                    Ok(LeanExpr::BVar(*index))
                } else {
                    index
                        .checked_add(amount)
                        .map(LeanExpr::BVar)
                        .ok_or_else(|| "Index overflow during lift".to_string())
                }
            }

            LeanExpr::Const(_, _) => Ok(self.clone()),

            LeanExpr::Pi(domain, body) => {
                // domain uses same cutoff
                let lifted_domain = domain.lift(amount, cutoff)?;
                // body uses cutoff + 1
                let new_cutoff = cutoff
                    .checked_add(1)
                    .ok_or_else(|| "Cutoff overflow during lift".to_string())?;
                let lifted_body = body.lift(amount, new_cutoff)?;

                Ok(LeanExpr::Pi(Box::new(lifted_domain), Box::new(lifted_body)))
            }

            LeanExpr::Lambda(parameter_type, body) => {
                let lifted_parameter_type = parameter_type.lift(amount, cutoff)?;
                let new_cutoff = cutoff
                    .checked_add(1)
                    .ok_or_else(|| "Cutoff overflow during lift".to_string())?;
                let lifted_body = body.lift(amount, new_cutoff)?;

                Ok(LeanExpr::Lambda(
                    Box::new(lifted_parameter_type),
                    Box::new(lifted_body),
                ))
            }

            LeanExpr::Application(function, arg) => {
                let lifted_function = function.lift(amount, cutoff)?;
                let lifted_arg = arg.lift(amount, cutoff)?;

                Ok(LeanExpr::Application(
                    Box::new(lifted_function),
                    Box::new(lifted_arg),
                ))
            }
        }
    }

    /// Removes the outermost binder #0 by replacing it with `replacement`.
    ///
    /// At recursive binder depth `d`:
    /// - `#index < d`  → belongs to an inner binder; unchanged
    /// - `#index == d` → this is the binder being removed;
    ///   use `replacement` lifted by `d`
    /// - `#index > d`  → refers outside the removed binder; decrement by 1
    pub fn instantiate_bvar0(&self, replacement: &LeanExpr) -> Result<LeanExpr, String> {
        self.instantiate_bvar0_at(replacement, 0)
    }

    fn instantiate_bvar0_at(
        &self,
        replacement: &LeanExpr,
        depth: usize,
    ) -> Result<LeanExpr, String> {
        match self {
            LeanExpr::Sort(level) => Ok(LeanExpr::Sort(level.clone())),

            LeanExpr::BVar(index) => {
                if *index < depth {
                    Ok(LeanExpr::BVar(*index))
                } else if *index == depth {
                    replacement.lift(depth, 0)
                } else {
                    index
                        .checked_sub(1)
                        .map(LeanExpr::BVar)
                        .ok_or_else(|| "Index underflow during instantiation".to_string())
                }
            }

            LeanExpr::Const(_, _) => Ok(self.clone()),

            LeanExpr::Pi(domain, body) => {
                let instantiated_domain = domain.instantiate_bvar0_at(replacement, depth)?;

                let new_depth = depth
                    .checked_add(1)
                    .ok_or_else(|| "Depth overflow during instantiation".to_string())?;
                let instantiated_body = body.instantiate_bvar0_at(replacement, new_depth)?;

                Ok(LeanExpr::Pi(
                    Box::new(instantiated_domain),
                    Box::new(instantiated_body),
                ))
            }

            LeanExpr::Lambda(parameter_type, body) => {
                let instantiated_parameter_type =
                    parameter_type.instantiate_bvar0_at(replacement, depth)?;

                let new_depth = depth
                    .checked_add(1)
                    .ok_or_else(|| "Depth overflow during instantiation".to_string())?;
                let instantiated_body = body.instantiate_bvar0_at(replacement, new_depth)?;

                Ok(LeanExpr::Lambda(
                    Box::new(instantiated_parameter_type),
                    Box::new(instantiated_body),
                ))
            }

            LeanExpr::Application(function, arg) => {
                let instantiated_function = function.instantiate_bvar0_at(replacement, depth)?;
                let instantiated_arg = arg.instantiate_bvar0_at(replacement, depth)?;

                Ok(LeanExpr::Application(
                    Box::new(instantiated_function),
                    Box::new(instantiated_arg),
                ))
            }
        }
    }
}
