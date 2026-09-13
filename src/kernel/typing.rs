use super::debruijn::lookup_bvar_type;
use super::{LeanDefinition, LeanExpr, Level, Name};

fn level_in_scope(level: &Level, level_params: &[Name]) -> bool {
    match level {
        Level::Zero => true,
        Level::Succ(inner) => level_in_scope(inner, level_params),
        Level::Param(name) => level_params.contains(name),
        Level::Max(u, v) => level_in_scope(u, level_params) && level_in_scope(v, level_params),
        Level::IMax(u, v) => level_in_scope(u, level_params) && level_in_scope(v, level_params),
    }
}

fn expr_levels_in_scope(expr: &LeanExpr, level_params: &[Name]) -> bool {
    match expr {
        LeanExpr::Sort(level) => level_in_scope(level, level_params),
        LeanExpr::BVar(_) => true,
        LeanExpr::Const(_, universes) => universes
            .iter()
            .all(|level| level_in_scope(level, level_params)),
        LeanExpr::Pi(domain, body)
        | LeanExpr::Lambda(domain, body)
        | LeanExpr::Application(domain, body) => {
            expr_levels_in_scope(domain, level_params) && expr_levels_in_scope(body, level_params)
        }
    }
}

pub(super) fn imax(u: &Level, v: &Level) -> Level {
    fn max_level(u: &Level, v: &Level) -> Level {
        match (u, v) {
            (Level::Zero, v) => v.clone(),
            (u, Level::Zero) => u.clone(),
            (u, v) if u == v => u.clone(),
            (Level::Succ(u_prev), Level::Succ(v_prev)) => {
                Level::Succ(Box::new(max_level(u_prev, v_prev)))
            }
            _ => Level::Max(Box::new(u.clone()), Box::new(v.clone())),
        }
    }

    match (u, v) {
        (_, Level::Zero) => Level::Zero,
        (Level::Zero, v) => v.clone(),
        (u, v) if u == v => u.clone(),
        (Level::Succ(u_prev), Level::Succ(v_prev)) => {
            Level::Succ(Box::new(max_level(u_prev, v_prev)))
        }
        _ => Level::IMax(Box::new(u.clone()), Box::new(v.clone())),
    }
}

impl LeanExpr {
    pub fn infer_type_with_env_and_context(
        &self,
        env: &[LeanDefinition],
        context: &[LeanExpr],
    ) -> Result<LeanExpr, String> {
        match self {
            LeanExpr::Sort(level) => {
                let next_level = Level::Succ(Box::new(level.clone()));
                Ok(LeanExpr::Sort(next_level))
            }
            LeanExpr::BVar(index) => lookup_bvar_type(context, *index),
            LeanExpr::Const(name, universes) => {
                if !universes.is_empty() {
                    return Err("Const with universe levels is not supported yet".to_string());
                }

                let definition = env
                    .iter()
                    .find(|def| &def.name == name)
                    .ok_or_else(|| format!("Constant {:?} not found in environment", name))?;

                Ok(definition.ty.clone())
            }
            LeanExpr::Pi(domain, body) => {
                let domain_type = domain.infer_type_with_env_and_context(env, context)?;

                let mut new_context = Vec::with_capacity(context.len() + 1);
                new_context.push(*domain.clone());
                new_context.extend_from_slice(context);
                let body_type = body.infer_type_with_env_and_context(env, &new_context)?;

                match (domain_type, body_type) {
                    (LeanExpr::Sort(level1), LeanExpr::Sort(level2)) => {
                        let result_level = imax(&level1, &level2);
                        Ok(LeanExpr::Sort(result_level))
                    }
                    _ => Err("Pi type inference failed: domain or body is not a Sort".to_string()),
                }
            }
            LeanExpr::Lambda(parameter_type, body) => {
                let parameter_type_type =
                    parameter_type.infer_type_with_env_and_context(env, context)?;

                // Should be Sort(_)
                if !matches!(parameter_type_type, LeanExpr::Sort(_)) {
                    return Err("Lambda parameter type is not a Sort".to_string());
                }

                let mut new_context = Vec::with_capacity(context.len() + 1);
                new_context.push(*parameter_type.clone());
                new_context.extend_from_slice(context);

                let body_type = body.infer_type_with_env_and_context(env, &new_context)?;

                Ok(LeanExpr::Pi(parameter_type.clone(), Box::new(body_type)))
            }
            LeanExpr::Application(function, arg) => {
                let function_type = function.infer_type_with_env_and_context(env, context)?;

                match function_type {
                    LeanExpr::Pi(domain, body) => {
                        let arg_type = arg.infer_type_with_env_and_context(env, context)?;

                        if *domain != arg_type {
                            return Err(format!(
                                "Application type mismatch: expected {:?}, got {:?}",
                                domain, arg_type
                            ));
                        }

                        body.instantiate_bvar0(arg)
                    }
                    _ => Err(
                        "Application type inference failed: function is not a Pi type".to_string(),
                    ),
                }
            }
        }
    }

    pub fn infer_type_with_context(&self, context: &[LeanExpr]) -> Result<LeanExpr, String> {
        self.infer_type_with_env_and_context(&[], context)
    }

    pub fn infer_type(&self) -> Result<LeanExpr, String> {
        self.infer_type_with_context(&[])
    }
}

impl LeanDefinition {
    pub fn check_with_env(&self, env: &[LeanDefinition]) -> Result<bool, String> {
        if !expr_levels_in_scope(&self.ty, &self.level_params)
            || !expr_levels_in_scope(&self.value, &self.level_params)
        {
            return Ok(false);
        }

        let inferred_ty_type = self.ty.infer_type_with_env_and_context(env, &[])?;

        if !matches!(inferred_ty_type, LeanExpr::Sort(_)) {
            return Ok(false);
        }

        let inferred_value_type = self.value.infer_type_with_env_and_context(env, &[])?;

        inferred_value_type.def_eq_with_env(&self.ty, env)
    }

    pub fn check(&self) -> Result<bool, String> {
        self.check_with_env(&[])
    }
}
