use super::{ExprTable, LevelTable, NameTable};
use crate::export::{
    DefinitionHints, DefinitionRecord, ExprBVarRecord, ExprForallRecord, ExprLamRecord,
    ExprSortRecord, LevelSuccRecord, NameNumRecord, NameStrRecord, Safety,
};
use crate::kernel::{LeanDefinition, LeanExpr, Level, Name};

impl NameTable {
    pub fn add_str(&mut self, record: &NameStrRecord) -> Result<(), String> {
        if record.id != self.names.len() {
            return Err(format!(
                "Non-sequential name record id: expected {}, got {}",
                self.names.len(),
                record.id
            ));
        }

        let prefix_name = self
            .names
            .get(record.name.prefix)
            .ok_or_else(|| format!("Invalid prefix index: {}", record.name.prefix))?
            .clone();

        self.names
            .push(Name::Str(Box::new(prefix_name), record.name.value.clone()));
        Ok(())
    }

    pub fn add_num(&mut self, record: &NameNumRecord) -> Result<(), String> {
        if record.id != self.names.len() {
            return Err(format!(
                "Non-sequential name record id: expected {}, got {}",
                self.names.len(),
                record.id
            ));
        }

        let prefix_name = self
            .names
            .get(record.name.prefix)
            .ok_or_else(|| format!("Invalid prefix index: {}", record.name.prefix))?
            .clone();

        self.names
            .push(Name::Num(Box::new(prefix_name), record.name.index));
        Ok(())
    }
}
impl LevelTable {
    pub fn add_succ(&mut self, record: &LevelSuccRecord) -> Result<(), String> {
        if record.id != self.levels.len() {
            return Err(format!(
                "Non-sequential level record id: expected {}, got {}",
                self.levels.len(),
                record.id
            ));
        }

        let prev_level = self
            .levels
            .get(record.predecessor)
            .ok_or_else(|| format!("Invalid previous level index: {}", record.predecessor))?
            .clone();

        self.levels.push(Level::Succ(Box::new(prev_level)));
        Ok(())
    }
}
impl ExprTable {
    pub fn add_sort(&mut self, record: &ExprSortRecord, levels: &LevelTable) -> Result<(), String> {
        if record.id != self.exprs.len() {
            return Err(format!(
                "Non-sequential expr record id: expected {}, got {}",
                self.exprs.len(),
                record.id
            ));
        }

        let level = levels
            .get(record.level)
            .ok_or_else(|| format!("Invalid level index: {}", record.level))?
            .clone();

        self.exprs.push(LeanExpr::Sort(level));
        Ok(())
    }

    pub fn add_forall(
        &mut self,
        record: &ExprForallRecord,
        names: &NameTable,
    ) -> Result<(), String> {
        if record.id != self.exprs.len() {
            return Err(format!(
                "Non-sequential expr record id: expected {}, got {}",
                self.exprs.len(),
                record.id
            ));
        }

        // record.forall_expr.name refers to an existing Name
        if names.names.get(record.forall_expr.name).is_none() {
            return Err(format!(
                "Invalid forall name index: {}",
                record.forall_expr.name
            ));
        }

        // record.forall_expr.type_id refers to an existing Expr
        if record.forall_expr.type_id >= self.exprs.len() {
            return Err(format!(
                "Invalid forall type expression index: {}",
                record.forall_expr.type_id
            ));
        }

        // record.forall_expr.body refers to an existing Expr
        if record.forall_expr.body >= self.exprs.len() {
            return Err(format!(
                "Invalid forall body expression index: {}",
                record.forall_expr.body
            ));
        }

        // append Pi(domain, body)
        let domain = self
            .get(record.forall_expr.type_id)
            .ok_or_else(|| {
                format!(
                    "Invalid forall type expression index: {}",
                    record.forall_expr.type_id
                )
            })?
            .clone();

        let body = self
            .get(record.forall_expr.body)
            .ok_or_else(|| {
                format!(
                    "Invalid forall body expression index: {}",
                    record.forall_expr.body
                )
            })?
            .clone();

        self.exprs
            .push(LeanExpr::Pi(Box::new(domain), Box::new(body)));
        Ok(())
    }

    pub fn add_bvar(&mut self, record: &ExprBVarRecord) -> Result<(), String> {
        if record.id != self.exprs.len() {
            return Err(format!(
                "Non-sequential expr record id: expected {}, got {}",
                self.exprs.len(),
                record.id
            ));
        }

        self.exprs.push(LeanExpr::BVar(record.index));
        Ok(())
    }

    pub fn add_lam(&mut self, record: &ExprLamRecord, names: &NameTable) -> Result<(), String> {
        if record.id != self.exprs.len() {
            return Err(format!(
                "Non-sequential expr record id: expected {}, got {}",
                self.exprs.len(),
                record.id
            ));
        }

        // record.lambda.name refers to an existing Name
        if names.names.get(record.lambda.name).is_none() {
            return Err(format!("Invalid lambda name index: {}", record.lambda.name));
        }

        // record.lambda.type_id refers to an existing Expr
        if record.lambda.type_id >= self.exprs.len() {
            return Err(format!(
                "Invalid lambda type expression index: {}",
                record.lambda.type_id
            ));
        }

        // record.lambda.body refers to an existing Expr
        if record.lambda.body >= self.exprs.len() {
            return Err(format!(
                "Invalid lambda body expression index: {}",
                record.lambda.body
            ));
        }

        // append Lambda(parameter_type, body)
        let parameter_type = self
            .get(record.lambda.type_id)
            .ok_or_else(|| {
                format!(
                    "Invalid lambda type expression index: {}",
                    record.lambda.type_id
                )
            })?
            .clone();

        let body = self
            .get(record.lambda.body)
            .ok_or_else(|| {
                format!(
                    "Invalid lambda body expression index: {}",
                    record.lambda.body
                )
            })?
            .clone();

        self.exprs
            .push(LeanExpr::Lambda(Box::new(parameter_type), Box::new(body)));
        Ok(())
    }
}
pub fn reconstruct_definition(
    record: &DefinitionRecord,
    names: &NameTable,
    exprs: &ExprTable,
) -> Result<LeanDefinition, String> {
    if !record.level_params.is_empty() {
        return Err("universe parameters are not supported yet".to_string());
    }

    if record.safety != Safety::Safe {
        return Err("non-safe definitions are not supported yet".to_string());
    }

    if record.hints != DefinitionHints::Keyword("opaque".to_string()) {
        return Err("definition hint is not supported yet".to_string());
    }

    let name = names
        .get(record.name)
        .ok_or_else(|| format!("Invalid name index: {}", record.name))?
        .clone();

    let ty = exprs
        .get(record.type_id)
        .ok_or_else(|| format!("Invalid type expression index: {}", record.type_id))?
        .clone();

    let value = exprs
        .get(record.value_id)
        .ok_or_else(|| format!("Invalid value expression index: {}", record.value_id))?
        .clone();

    Ok(LeanDefinition { name, ty, value })
}
