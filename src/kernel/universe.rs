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

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LevelAtom {
    Param(Name),
    IMax(Box<Level>, Box<Level>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelNormalForm {
    pub offset: usize,
    pub terms: BTreeMap<LevelAtom, usize>,
}

impl LevelNormalForm {
    pub fn zero() -> Self {
        Self {
            offset: 0,
            terms: BTreeMap::new(),
        }
    }

    pub fn is_zero(&self) -> bool {
        self.offset == 0 && self.terms.is_empty()
    }

    pub fn is_never_zero(&self) -> bool {
        self.offset >= 1 || (!self.terms.is_empty() && self.terms.values().all(|&k| k >= 1))
    }

    fn clean(&mut self) {
        if self.terms.values().any(|&k| k >= self.offset) {
            self.offset = 0;
        }
    }

    pub fn succ(mut self) -> Self {
        self.offset += 1;
        for k in self.terms.values_mut() {
            *k += 1;
        }
        self.clean();
        self
    }

    pub fn max(u: Self, v: Self) -> Self {
        let offset = std::cmp::max(u.offset, v.offset);
        let mut terms = u.terms;
        for (atom, k) in v.terms {
            terms
                .entry(atom)
                .and_modify(|existing| *existing = std::cmp::max(*existing, k))
                .or_insert(k);
        }
        let mut res = Self { offset, terms };
        res.clean();
        res
    }

    pub fn components(&self) -> Vec<Self> {
        let mut parts = Vec::new();
        if self.offset > 0 || self.terms.is_empty() {
            parts.push(Self {
                offset: self.offset,
                terms: BTreeMap::new(),
            });
        }
        for (atom, &k) in &self.terms {
            let mut terms = BTreeMap::new();
            terms.insert(atom.clone(), k);
            parts.push(Self { offset: 0, terms });
        }
        parts
    }

    pub fn imax(u: Self, v: Self) -> Self {
        if v.is_zero() {
            return Self::zero();
        }
        if u.is_zero() {
            return v;
        }
        if u == v {
            return u;
        }
        if v.is_never_zero() {
            return Self::max(u, v);
        }

        let v_comps = v.components();
        if v_comps.len() > 1 {
            let mut iter = v_comps.into_iter();
            let first = iter.next().unwrap();
            let first_imax = Self::imax(u.clone(), first);
            return iter.fold(first_imax, |acc, next| {
                Self::max(acc, Self::imax(u.clone(), next))
            });
        }

        let u_comps = u.components();
        if u_comps.len() > 1 {
            let mut iter = u_comps.into_iter();
            let first = iter.next().unwrap();
            let first_imax = Self::imax(first, v.clone());
            return iter.fold(first_imax, |acc, next| {
                Self::max(acc, Self::imax(next, v.clone()))
            });
        }

        let atom = LevelAtom::IMax(Box::new(u.to_level()), Box::new(v.to_level()));
        let mut terms = BTreeMap::new();
        terms.insert(atom, 0);
        Self { offset: 0, terms }
    }

    pub fn from_level(level: &Level) -> Self {
        match level {
            Level::Zero => Self::zero(),
            Level::Succ(inner) => Self::from_level(inner).succ(),
            Level::Param(name) => {
                let mut terms = BTreeMap::new();
                terms.insert(LevelAtom::Param(name.clone()), 0);
                Self { offset: 0, terms }
            }
            Level::Max(a, b) => Self::max(Self::from_level(a), Self::from_level(b)),
            Level::IMax(a, b) => Self::imax(Self::from_level(a), Self::from_level(b)),
        }
    }

    pub fn to_level(&self) -> Level {
        let mut parts = Vec::new();
        if self.offset > 0 || self.terms.is_empty() {
            let mut c_lvl = Level::Zero;
            for _ in 0..self.offset {
                c_lvl = Level::Succ(Box::new(c_lvl));
            }
            parts.push(c_lvl);
        }

        for (atom, &k) in &self.terms {
            let mut lvl = match atom {
                LevelAtom::Param(name) => Level::Param(name.clone()),
                LevelAtom::IMax(u, v) => Level::IMax(u.clone(), v.clone()),
            };
            for _ in 0..k {
                lvl = Level::Succ(Box::new(lvl));
            }
            parts.push(lvl);
        }

        if parts.is_empty() {
            Level::Zero
        } else {
            let mut iter = parts.into_iter();
            let first = iter.next().unwrap();
            iter.fold(first, |acc, next| Level::Max(Box::new(acc), Box::new(next)))
        }
    }
}

impl Level {
    pub fn is_equiv(&self, other: &Level) -> bool {
        LevelNormalForm::from_level(self) == LevelNormalForm::from_level(other)
    }

    pub fn normalize(&self) -> Level {
        LevelNormalForm::from_level(self).to_level()
    }
}
