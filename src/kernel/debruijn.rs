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
        }
    }
}
