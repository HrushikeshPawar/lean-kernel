use crate::kernel::{LeanDeclaration, LeanExpr};

impl LeanExpr {
    pub fn whnf_with_env(&self, env: &[LeanDeclaration]) -> Result<LeanExpr, String> {
        match self {
            LeanExpr::Sort(_) => Ok(self.clone()),
            LeanExpr::BVar(_) => Ok(self.clone()),
            LeanExpr::Pi(_, _) => Ok(self.clone()),
            LeanExpr::Lambda(_, _) => Ok(self.clone()),
            LeanExpr::Const(name, universes) => match env.iter().find(|decl| decl.name() == name) {
                Some(LeanDeclaration::Definition(definition)) => {
                    let instantiated_value = definition
                        .value
                        .instantiate_level_params(&definition.level_params, universes)
                        .map_err(|error| error.to_string())?;

                    instantiated_value.whnf_with_env(env)
                }
                // Axioms are neutral: they never unfold.
                Some(LeanDeclaration::Axiom(_)) => Ok(self.clone()),
                None => Ok(self.clone()),
            },
            LeanExpr::Application(function, arg) => {
                let reduced_function = function.whnf_with_env(env)?;
                match reduced_function {
                    LeanExpr::Lambda(_, body) => {
                        let instantiated_body = body.instantiate_bvar0(arg)?;
                        instantiated_body.whnf_with_env(env)
                    }
                    _ => Ok(LeanExpr::Application(
                        Box::new(reduced_function),
                        arg.clone(),
                    )),
                }
            }
        }
    }

    pub fn def_eq_with_env(
        &self,
        other: &LeanExpr,
        env: &[LeanDeclaration],
    ) -> Result<bool, String> {
        let self_whnf = self.whnf_with_env(env)?;
        let other_whnf = other.whnf_with_env(env)?;

        if self == other {
            return Ok(true);
        }

        match (self_whnf, other_whnf) {
            (LeanExpr::Sort(level1), LeanExpr::Sort(level2)) => Ok(level1 == level2),
            (LeanExpr::BVar(index1), LeanExpr::BVar(index2)) => Ok(index1 == index2),
            (LeanExpr::Const(name1, levels1), LeanExpr::Const(name2, levels2)) => {
                if name1 != name2 || levels1.len() != levels2.len() {
                    return Ok(false);
                }
                for (level1, level2) in levels1.iter().zip(levels2.iter()) {
                    if level1 != level2 {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            (LeanExpr::Pi(domain1, body1), LeanExpr::Pi(domain2, body2)) => {
                if !domain1.def_eq_with_env(&domain2, env)? {
                    return Ok(false);
                }
                body1.def_eq_with_env(&body2, env)
            }
            (
                LeanExpr::Lambda(parameter_type1, body1),
                LeanExpr::Lambda(parameter_type2, body2),
            ) => {
                if !parameter_type1.def_eq_with_env(&parameter_type2, env)? {
                    return Ok(false);
                }
                body1.def_eq_with_env(&body2, env)
            }
            (LeanExpr::Application(function1, arg1), LeanExpr::Application(function2, arg2)) => {
                if !function1.def_eq_with_env(&function2, env)? {
                    return Ok(false);
                }
                arg1.def_eq_with_env(&arg2, env)
            }
            _ => Ok(false),
        }
    }
}
