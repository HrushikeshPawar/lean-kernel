use super::debruijn::lookup_bvar_type;
use super::{LeanDefinition, LeanExpr, Level};

pub(super) fn imax(u: &Level, v: &Level) -> Level {
    fn max_level(u: &Level, v: &Level) -> Level {
        match (u, v) {
            (Level::Zero, v) => v.clone(),
            (u, Level::Zero) => u.clone(),
            (Level::Succ(u_prev), Level::Succ(v_prev)) => {
                Level::Succ(Box::new(max_level(u_prev, v_prev)))
            }
        }
    }

    match v {
        Level::Zero => Level::Zero,
        _ => max_level(u, v),
    }
}

impl LeanExpr {
    pub fn infer_type_with_context(&self, context: &[LeanExpr]) -> Result<LeanExpr, String> {
        match self {
            LeanExpr::Sort(level) => {
                let next_level = Level::Succ(Box::new(level.clone()));
                Ok(LeanExpr::Sort(next_level))
            }
            LeanExpr::Pi(domain, body) => {
                let domain_type = domain.infer_type_with_context(context)?;

                let mut new_context = Vec::with_capacity(context.len() + 1);
                new_context.push(*domain.clone());
                new_context.extend_from_slice(context);
                let body_type = body.infer_type_with_context(&new_context)?;

                match (domain_type, body_type) {
                    (LeanExpr::Sort(level1), LeanExpr::Sort(level2)) => {
                        let result_level = imax(&level1, &level2);
                        Ok(LeanExpr::Sort(result_level))
                    }
                    _ => Err("Pi type inference failed: domain or body is not a Sort".to_string()),
                }
            }
            LeanExpr::BVar(index) => lookup_bvar_type(context, *index),
            LeanExpr::Lambda(parameter_type, body) => {
                let parameter_type_type = parameter_type.infer_type_with_context(context)?;

                // Should be Sort(_)
                if !matches!(parameter_type_type, LeanExpr::Sort(_)) {
                    return Err("Lambda parameter type is not a Sort".to_string());
                }

                let mut new_context = Vec::with_capacity(context.len() + 1);
                new_context.push(*parameter_type.clone());
                new_context.extend_from_slice(context);

                let body_type = body.infer_type_with_context(&new_context)?;

                Ok(LeanExpr::Pi(parameter_type.clone(), Box::new(body_type)))
            }
        }
    }

    pub fn infer_type(&self) -> Result<LeanExpr, String> {
        self.infer_type_with_context(&[])
    }
}

impl LeanDefinition {
    pub fn check(&self) -> Result<bool, String> {
        // 1. Infer the type of self.ty.
        let inferred_ty_type = self.ty.infer_type()?;

        // 2. Make sure that inferred result is a LeanExpr::Sort(_).
        //    If it isn't, the declaration's annotation is not itself a type.
        if !matches!(inferred_ty_type, LeanExpr::Sort(_)) {
            return Ok(false);
        }
        // 3. Infer the type of self.value.
        let inferred_value_type = self.value.infer_type()?;

        // 4. Compare that inferred type with self.ty.
        // Return Ok(true) / Ok(false).
        Ok(inferred_value_type == self.ty)
    }
}
