use super::LeanExpr;
use super::Level;
use super::Name;
use super::typing::TypeError;

impl LeanExpr {
    /// Replaces every universe `Param(uᵢ)` in `params` with the corresponding
    /// supplied level in `levels`, throughout the expression.
    ///
    /// Universe parameters are a separate namespace from de Bruijn binders,
    /// so no shifting or substitution of `BVar`s happens here.
    pub(crate) fn instantiate_level_params(
        &self,
        params: &[Name],
        levels: &[Level],
    ) -> Result<LeanExpr, TypeError> {
        if params.len() != levels.len() {
            return Err(TypeError::Invalid(format!(
                "Universe argument count mismatch: expected {}, got {}",
                params.len(),
                levels.len()
            )));
        }

        fn instantiate_level(level: &Level, params: &[Name], levels: &[Level]) -> Level {
            match level {
                Level::Zero => Level::Zero,
                Level::Succ(inner) => {
                    Level::Succ(Box::new(instantiate_level(inner, params, levels)))
                }
                Level::Param(p) => params
                    .iter()
                    .position(|u| u == p)
                    .map(|index| levels[index].clone())
                    .unwrap_or_else(|| level.clone()),
                Level::Max(a, b) => Level::Max(
                    Box::new(instantiate_level(a, params, levels)),
                    Box::new(instantiate_level(b, params, levels)),
                ),
                Level::IMax(a, b) => Level::IMax(
                    Box::new(instantiate_level(a, params, levels)),
                    Box::new(instantiate_level(b, params, levels)),
                ),
            }
        }

        fn instantiate_expr(
            expr: &LeanExpr,
            params: &[Name],
            levels: &[Level],
        ) -> Result<LeanExpr, String> {
            match expr {
                LeanExpr::Sort(level) => {
                    Ok(LeanExpr::Sort(instantiate_level(level, params, levels)))
                }
                LeanExpr::BVar(_) => Ok(expr.clone()),
                LeanExpr::Const(name, us) => Ok(LeanExpr::Const(
                    name.clone(),
                    us.iter()
                        .map(|level| instantiate_level(level, params, levels))
                        .collect(),
                )),
                LeanExpr::Pi(domain, body) => Ok(LeanExpr::Pi(
                    Box::new(instantiate_expr(domain, params, levels)?),
                    Box::new(instantiate_expr(body, params, levels)?),
                )),
                LeanExpr::Lambda(parameter_type, body) => Ok(LeanExpr::Lambda(
                    Box::new(instantiate_expr(parameter_type, params, levels)?),
                    Box::new(instantiate_expr(body, params, levels)?),
                )),
                LeanExpr::Application(function, arg) => Ok(LeanExpr::Application(
                    Box::new(instantiate_expr(function, params, levels)?),
                    Box::new(instantiate_expr(arg, params, levels)?),
                )),
            }
        }

        instantiate_expr(self, params, levels).map_err(TypeError::from)
    }
}
