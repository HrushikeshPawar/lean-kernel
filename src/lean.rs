use crate::export::ExprForallRecord;
use crate::export::ExprSortRecord;
use crate::export::LevelSuccRecord;
use crate::export::{DefinitionHints, DefinitionRecord, Safety};
use crate::export::{NameNumRecord, NameStrRecord};

#[derive(Debug, Clone, PartialEq)]
pub enum Name {
    Anonymous,
    Str(Box<Name>, String),
    Num(Box<Name>, usize),
}

#[derive(Debug)]
pub struct NameTable {
    names: Vec<Name>,
}

impl NameTable {
    pub fn new() -> Self {
        NameTable {
            names: vec![Name::Anonymous],
        }
    }

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

    pub fn get(&self, index: usize) -> Option<&Name> {
        self.names.get(index)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Level {
    Zero,
    Succ(Box<Level>),
}

#[derive(Debug)]
pub struct LevelTable {
    levels: Vec<Level>,
}

impl LevelTable {
    pub fn new() -> Self {
        LevelTable {
            levels: vec![Level::Zero],
        }
    }

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

    pub fn get(&self, id: usize) -> Option<&Level> {
        self.levels.get(id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LeanExpr {
    Sort(Level),
    Pi(Box<LeanExpr>, Box<LeanExpr>),
}

#[derive(Debug)]
pub struct ExprTable {
    exprs: Vec<LeanExpr>,
}

fn imax(u: &Level, v: &Level) -> Level {
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
    pub fn infer_type(&self) -> Result<LeanExpr, String> {
        match self {
            LeanExpr::Sort(level) => {
                let next_level = Level::Succ(Box::new(level.clone()));
                Ok(LeanExpr::Sort(next_level))
            }
            LeanExpr::Pi(domain, body) => {
                let domain_type = domain.infer_type()?;
                let body_type = body.infer_type()?;

                match (domain_type, body_type) {
                    (LeanExpr::Sort(level1), LeanExpr::Sort(level2)) => {
                        let result_level = imax(&level1, &level2);
                        Ok(LeanExpr::Sort(result_level))
                    }
                    _ => Err("Pi type inference failed: domain or body is not a Sort".to_string()),
                }
            }
        }
    }
}

impl ExprTable {
    pub fn new() -> Self {
        ExprTable { exprs: Vec::new() }
    }

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

    pub fn get(&self, id: usize) -> Option<&LeanExpr> {
        self.exprs.get(id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeanDefinition {
    pub name: Name,
    pub ty: LeanExpr,
    pub value: LeanExpr,
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

#[derive(Debug)]
pub struct ImportState {
    pub names: NameTable,
    pub levels: LevelTable,
    pub exprs: ExprTable,
}

impl ImportState {
    pub fn new() -> Self {
        Self {
            names: NameTable::new(),
            levels: LevelTable::new(),
            exprs: ExprTable::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::{BinderInfo, ExprForallPayload, ExprForallRecord};
    use crate::export::{NameNumPayload, NameNumRecord};
    use crate::export::{NameStrPayload, NameStrRecord};

    #[test]
    fn name_table_starts_with_anonymous_name() {
        let table = NameTable::new();

        assert_eq!(table.get(0), Some(&Name::Anonymous));
        assert_eq!(table.get(1), None);
    }

    #[test]
    fn reconstructs_basic_def_name() {
        let mut table = NameTable::new();

        let record = NameStrRecord {
            id: 1,
            name: NameStrPayload {
                prefix: 0,
                value: "basicDef".to_string(),
            },
        };

        assert_eq!(table.add_str(&record), Ok(()));

        assert_eq!(
            table.get(1),
            Some(&Name::Str(
                Box::new(Name::Anonymous),
                "basicDef".to_string()
            ))
        );
    }

    #[test]
    fn name_record_rejects_unknown_prefix() {
        let mut table = NameTable::new();

        let record = NameStrRecord {
            id: 1,
            name: NameStrPayload {
                prefix: 99,
                value: "bad".to_string(),
            },
        };

        assert!(table.add_str(&record).is_err());
    }

    #[test]
    fn name_record_rejects_non_sequential_id() {
        let mut table = NameTable::new();

        let record = NameStrRecord {
            id: 7,
            name: NameStrPayload {
                prefix: 0,
                value: "bad".to_string(),
            },
        };

        assert!(table.add_str(&record).is_err());
    }

    #[test]
    fn reconstructs_nested_name() {
        let mut table = NameTable::new();

        let foo = NameStrRecord {
            id: 1,
            name: NameStrPayload {
                prefix: 0,
                value: "Foo".to_string(),
            },
        };

        let bar = NameStrRecord {
            id: 2,
            name: NameStrPayload {
                prefix: 1,
                value: "Bar".to_string(),
            },
        };

        assert_eq!(table.add_str(&foo), Ok(()));
        assert_eq!(table.add_str(&bar), Ok(()));

        assert_eq!(
            table.get(2),
            Some(&Name::Str(
                Box::new(Name::Str(Box::new(Name::Anonymous), "Foo".to_string(),)),
                "Bar".to_string(),
            ))
        );
    }

    #[test]
    fn name_table_adds_num_name() {
        let mut table = NameTable::new();

        let record = NameNumRecord {
            id: 1,
            name: NameNumPayload {
                prefix: 0,
                index: 7,
            },
        };

        table.add_num(&record).unwrap();

        assert_eq!(table.get(1), Some(&Name::Num(Box::new(Name::Anonymous), 7)));
    }

    #[test]
    fn name_table_rejects_non_sequential_num_id() {
        let mut table = NameTable::new();

        let record = NameNumRecord {
            id: 2,
            name: NameNumPayload {
                prefix: 0,
                index: 7,
            },
        };

        let error = table.add_num(&record).unwrap_err();

        assert!(error.contains("Non-sequential name record id"));
    }

    #[test]
    fn name_table_rejects_invalid_num_prefix() {
        let mut table = NameTable::new();

        let record = NameNumRecord {
            id: 1,
            name: NameNumPayload {
                prefix: 42,
                index: 7,
            },
        };

        let error = table.add_num(&record).unwrap_err();

        assert!(error.contains("Invalid prefix index: 42"));
    }

    #[test]
    fn level_table_starts_with_zero() {
        let table = LevelTable::new();

        assert_eq!(table.get(0), Some(&Level::Zero));
        assert_eq!(table.get(1), None);
    }

    #[test]
    fn reconstructs_successor_of_zero() {
        let mut table = LevelTable::new();

        let record = LevelSuccRecord {
            id: 1,
            predecessor: 0,
        };

        assert_eq!(table.add_succ(&record), Ok(()));

        assert_eq!(table.get(1), Some(&Level::Succ(Box::new(Level::Zero))));
    }

    #[test]
    fn level_record_rejects_unknown_predecessor() {
        let mut table = LevelTable::new();

        let record = LevelSuccRecord {
            id: 1,
            predecessor: 99,
        };

        assert!(table.add_succ(&record).is_err());
    }

    #[test]
    fn level_record_rejects_non_sequential_id() {
        let mut table = LevelTable::new();

        let record = LevelSuccRecord {
            id: 5,
            predecessor: 0,
        };

        assert!(table.add_succ(&record).is_err());
    }

    #[test]
    fn expr_table_starts_empty() {
        let table = ExprTable::new();

        assert_eq!(table.get(0), None);
    }

    #[test]
    fn reconstructs_type_sort() {
        let mut levels = LevelTable::new();

        let level_record = LevelSuccRecord {
            id: 1,
            predecessor: 0,
        };

        assert_eq!(levels.add_succ(&level_record), Ok(()));

        let mut exprs = ExprTable::new();

        let expr_record = ExprSortRecord { id: 0, level: 1 };

        assert_eq!(exprs.add_sort(&expr_record, &levels), Ok(()));

        assert_eq!(
            exprs.get(0),
            Some(&LeanExpr::Sort(Level::Succ(Box::new(Level::Zero))))
        );
    }

    #[test]
    fn reconstructs_prop_sort() {
        let levels = LevelTable::new();
        let mut exprs = ExprTable::new();

        let expr_record = ExprSortRecord { id: 0, level: 0 };

        assert_eq!(exprs.add_sort(&expr_record, &levels), Ok(()));

        assert_eq!(exprs.get(0), Some(&LeanExpr::Sort(Level::Zero)));
    }

    #[test]
    fn sort_record_rejects_unknown_level() {
        let levels = LevelTable::new();
        let mut exprs = ExprTable::new();

        let expr_record = ExprSortRecord { id: 0, level: 99 };

        assert!(exprs.add_sort(&expr_record, &levels).is_err());
    }

    #[test]
    fn sort_record_rejects_non_sequential_id() {
        let levels = LevelTable::new();
        let mut exprs = ExprTable::new();

        let expr_record = ExprSortRecord { id: 7, level: 0 };

        assert!(exprs.add_sort(&expr_record, &levels).is_err());
    }

    #[test]
    fn reconstructs_basic_def_expression_table() {
        let mut levels = LevelTable::new();

        assert_eq!(
            levels.add_succ(&LevelSuccRecord {
                id: 1,
                predecessor: 0,
            }),
            Ok(())
        );

        let mut exprs = ExprTable::new();

        assert_eq!(
            exprs.add_sort(&ExprSortRecord { id: 0, level: 1 }, &levels),
            Ok(())
        );

        assert_eq!(
            exprs.add_sort(&ExprSortRecord { id: 1, level: 0 }, &levels),
            Ok(())
        );

        assert_eq!(
            exprs.get(0),
            Some(&LeanExpr::Sort(Level::Succ(Box::new(Level::Zero))))
        );

        assert_eq!(exprs.get(1), Some(&LeanExpr::Sort(Level::Zero)));
    }

    #[test]
    fn reconstructs_nested_successor_level() {
        let mut table = LevelTable::new();

        assert_eq!(
            table.add_succ(&LevelSuccRecord {
                id: 1,
                predecessor: 0,
            }),
            Ok(())
        );

        assert_eq!(
            table.add_succ(&LevelSuccRecord {
                id: 2,
                predecessor: 1,
            }),
            Ok(())
        );

        assert_eq!(
            table.get(2),
            Some(&Level::Succ(Box::new(Level::Succ(Box::new(Level::Zero)))))
        );
    }

    #[test]
    fn reconstructs_basic_def_declaration() {
        // Build NameTable:
        //   0 = anonymous
        //   1 = basicDef
        let mut name_table = NameTable::new();
        let name_record = NameStrRecord {
            id: 1,
            name: NameStrPayload {
                prefix: 0,
                value: "basicDef".to_string(),
            },
        };
        assert_eq!(name_table.add_str(&name_record), Ok(()));

        // Build LevelTable:
        //   0 = Zero
        //   1 = Succ(Zero)
        let mut level_table = LevelTable::new();
        let level_record = LevelSuccRecord {
            id: 1,
            predecessor: 0,
        };
        assert_eq!(level_table.add_succ(&level_record), Ok(()));

        // Build ExprTable:
        //   0 = Sort(Succ(Zero))
        //   1 = Sort(Zero)
        let mut expr_table = ExprTable::new();
        let expr_record_0 = ExprSortRecord { id: 0, level: 1 };
        let expr_record_1 = ExprSortRecord { id: 1, level: 0 };
        assert_eq!(expr_table.add_sort(&expr_record_0, &level_table), Ok(()));
        assert_eq!(expr_table.add_sort(&expr_record_1, &level_table), Ok(()));

        // Construct the real DefinitionRecord from basicDef.
        let definition_record = DefinitionRecord {
            all: vec![1],
            name: 1,
            type_id: 0,
            value_id: 1,
            hints: DefinitionHints::Keyword("opaque".to_string()),
            level_params: vec![],
            safety: Safety::Safe,
        };

        // Call reconstruct_definition(...)
        let definition = reconstruct_definition(&definition_record, &name_table, &expr_table)
            .expect("Failed to reconstruct definition");

        // Assert:
        //   name  == basicDef
        //   ty    == Sort(Succ(Zero))
        //   value == Sort(Zero)
        assert_eq!(
            definition.name,
            Name::Str(Box::new(Name::Anonymous), "basicDef".to_string())
        );

        assert_eq!(
            definition.ty,
            LeanExpr::Sort(Level::Succ(Box::new(Level::Zero)))
        );

        assert_eq!(definition.value, LeanExpr::Sort(Level::Zero));
    }

    #[test]
    fn prop_has_type_type() {
        let prop = LeanExpr::Sort(Level::Zero);

        assert_eq!(
            prop.infer_type(),
            Ok(LeanExpr::Sort(Level::Succ(Box::new(Level::Zero))))
        );
    }

    #[test]
    fn type_has_next_universe_as_type() {
        let type_zero = LeanExpr::Sort(Level::Succ(Box::new(Level::Zero)));

        assert_eq!(
            type_zero.infer_type(),
            Ok(LeanExpr::Sort(Level::Succ(Box::new(Level::Succ(
                Box::new(Level::Zero)
            )))))
        );
    }

    #[test]
    fn basic_def_is_valid() {
        let definition = LeanDefinition {
            name: Name::Str(Box::new(Name::Anonymous), "basicDef".to_string()),
            ty: LeanExpr::Sort(Level::Succ(Box::new(Level::Zero))),
            value: LeanExpr::Sort(Level::Zero),
        };

        assert_eq!(definition.check(), Ok(true));
    }

    #[test]
    fn definition_rejects_value_with_wrong_type() {
        let definition = LeanDefinition {
            name: Name::Str(Box::new(Name::Anonymous), "bad".to_string()),

            // Claims: bad : Prop
            ty: LeanExpr::Sort(Level::Zero),

            // But Prop : Type, not Prop.
            value: LeanExpr::Sort(Level::Zero),
        };

        assert_eq!(definition.check(), Ok(false));
    }

    #[test]
    fn reconstructs_forall_expression() {
        let mut names = NameTable::new();

        names
            .add_str(&NameStrRecord {
                id: 1,
                name: NameStrPayload {
                    prefix: 0,
                    value: "x".to_string(),
                },
            })
            .unwrap();

        let levels = LevelTable::new();
        let mut exprs = ExprTable::new();

        exprs
            .add_sort(&ExprSortRecord { id: 0, level: 0 }, &levels)
            .unwrap();

        let record = ExprForallRecord {
            id: 1,
            forall_expr: ExprForallPayload {
                name: 1,
                type_id: 0,
                body: 0,
                binder_info: BinderInfo::Default,
            },
        };

        exprs.add_forall(&record, &names).unwrap();

        assert_eq!(
            exprs.get(1),
            Some(&LeanExpr::Pi(
                Box::new(LeanExpr::Sort(Level::Zero)),
                Box::new(LeanExpr::Sort(Level::Zero)),
            ))
        );
    }

    #[test]
    fn forall_expression_rejects_unknown_binder_name() {
        let names = NameTable::new();
        let levels = LevelTable::new();
        let mut exprs = ExprTable::new();

        exprs
            .add_sort(&ExprSortRecord { id: 0, level: 0 }, &levels)
            .unwrap();

        let record = ExprForallRecord {
            id: 1,
            forall_expr: ExprForallPayload {
                name: 99,
                type_id: 0,
                body: 0,
                binder_info: BinderInfo::Default,
            },
        };

        assert!(exprs.add_forall(&record, &names).is_err());
    }

    #[test]
    fn forall_expression_rejects_unknown_type_expression() {
        let mut names = NameTable::new();

        names
            .add_str(&NameStrRecord {
                id: 1,
                name: NameStrPayload {
                    prefix: 0,
                    value: "x".to_string(),
                },
            })
            .unwrap();

        let mut exprs = ExprTable::new();

        let record = ExprForallRecord {
            id: 0,
            forall_expr: ExprForallPayload {
                name: 1,
                type_id: 99,
                body: 99,
                binder_info: BinderInfo::Default,
            },
        };

        assert!(exprs.add_forall(&record, &names).is_err());
    }

    #[test]
    fn prop_to_prop_pi_has_type_type() {
        let prop = LeanExpr::Sort(Level::Zero);

        let pi = LeanExpr::Pi(Box::new(prop.clone()), Box::new(prop));

        assert_eq!(
            pi.infer_type(),
            Ok(LeanExpr::Sort(Level::Succ(Box::new(Level::Zero))))
        );
    }

    #[test]
    fn imax_is_zero_when_second_level_is_zero() {
        let one = Level::Succ(Box::new(Level::Zero));

        assert_eq!(imax(&one, &Level::Zero), Level::Zero);
    }

    #[test]
    fn imax_uses_max_when_second_level_is_nonzero() {
        let one = Level::Succ(Box::new(Level::Zero));
        let two = Level::Succ(Box::new(one.clone()));

        assert_eq!(imax(&two, &one), two);
    }
}
