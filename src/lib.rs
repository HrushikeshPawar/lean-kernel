use std::cmp::max;
#[derive(Debug, PartialEq, Clone)]
pub enum Term {
    Index(usize),
    Lambda(Type, Box<Term>),
    Application(Box<Term>, Box<Term>),
}

impl Term {
    pub fn count_nodes(&self) -> usize {
        match self {
            Term::Index(_) => 1,
            Term::Lambda(_, body) => 1 + body.count_nodes(),
            Term::Application(func, arg) => 1 + func.count_nodes() + arg.count_nodes(),
        }
    }

    pub fn shift(&self, delta: isize, cutoff: usize) -> Option<Term> {
        match self {
            Term::Index(value) => {
                if value < &cutoff {
                    Some(Term::Index(*value))
                } else {
                    let new_val = value.checked_add_signed(delta)?;
                    Some(Term::Index(new_val))
                }
            }
            Term::Lambda(parameter_type, body) => {
                let new_body = body.shift(delta, cutoff.checked_add(1)?)?;
                Some(Term::Lambda(parameter_type.clone(), Box::new(new_body)))
            }
            Term::Application(func, arg) => Some(Term::Application(
                Box::new(func.shift(delta, cutoff)?),
                Box::new(arg.shift(delta, cutoff)?),
            )),
        }
    }

    pub fn substitute(&self, target: usize, replacement: &Term) -> Option<Term> {
        match self {
            Term::Index(value) => {
                if *value == target {
                    Some(replacement.clone())
                } else {
                    Some(Term::Index(*value))
                }
            }
            Term::Lambda(parameter_type, body) => {
                let shifted_replacement = replacement.shift(1, 0)?;
                let new_body = body.substitute(target.checked_add(1)?, &shifted_replacement)?;
                Some(Term::Lambda(parameter_type.clone(), Box::new(new_body)))
            }
            Term::Application(func, arg) => Some(Term::Application(
                Box::new(func.substitute(target, replacement)?),
                Box::new(arg.substitute(target, replacement)?),
            )),
        }
    }

    pub fn beta_reduce(&self) -> Option<Term> {
        match self {
            Term::Application(func, arg) => {
                if let Term::Lambda(_, body) = &**func {
                    let shifted_arg = arg.shift(1, 0)?;
                    let substituted_body = body.substitute(0, &shifted_arg)?;
                    let result = substituted_body.shift(-1, 0)?;
                    Some(result)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn infer_type(&self, context: &[Type]) -> Option<Type> {
        match self {
            Term::Index(val) => lookup_type(context, *val),
            Term::Lambda(parameter_type, body) => {
                let mut temp_context = Vec::with_capacity(context.len() + 1);
                temp_context.push(parameter_type.clone());
                temp_context.extend_from_slice(context);
                Some(Type::Arrow(
                    Box::new(parameter_type.clone()),
                    Box::new(body.infer_type(&temp_context)?),
                ))
            }
            Term::Application(func, arg) => {
                let func_type = func.infer_type(context)?;
                let arg_type = arg.infer_type(context)?;

                match func_type {
                    Type::Arrow(input_type, output_type) => {
                        if *input_type != arg_type {
                            None
                        } else {
                            Some(*output_type)
                        }
                    }
                    _ => None,
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Base,
    Arrow(Box<Type>, Box<Type>),
}

pub fn lookup_type(context: &[Type], index: usize) -> Option<Type> {
    Some(context.get(index)?.clone())
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Sort(usize),
    Index(usize),
    Const(usize),
    Pi(Box<Expr>, Box<Expr>),
    Lambda(Box<Expr>, Box<Expr>),
    Application(Box<Expr>, Box<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Declaration {
    pub ty: Expr,
    pub value: Option<Expr>,
}

impl Expr {
    pub fn shift(&self, delta: isize, cutoff: usize) -> Option<Expr> {
        match self {
            Expr::Sort(u) => Some(Expr::Sort(*u)),
            Expr::Index(value) => {
                if value < &cutoff {
                    Some(Expr::Index(*value))
                } else {
                    let new_val = value.checked_add_signed(delta)?;
                    Some(Expr::Index(new_val))
                }
            }
            Expr::Const(_) => Some(self.clone()),
            Expr::Pi(domain, codomain) => {
                let new_codomain = codomain.shift(delta, cutoff.checked_add(1)?)?;
                Some(Expr::Pi(
                    Box::new(domain.shift(delta, cutoff)?),
                    Box::new(new_codomain),
                ))
            }
            Expr::Lambda(parameter_type, body) => {
                let new_body = body.shift(delta, cutoff.checked_add(1)?)?;
                Some(Expr::Lambda(
                    Box::new(parameter_type.shift(delta, cutoff)?),
                    Box::new(new_body),
                ))
            }
            Expr::Application(func, arg) => Some(Expr::Application(
                Box::new(func.shift(delta, cutoff)?),
                Box::new(arg.shift(delta, cutoff)?),
            )),
        }
    }

    pub fn substitute(&self, target: usize, replacement: &Expr) -> Option<Expr> {
        match self {
            Expr::Sort(u) => Some(Expr::Sort(*u)),
            Expr::Index(value) => {
                if *value == target {
                    Some(replacement.clone())
                } else {
                    Some(Expr::Index(*value))
                }
            }
            Expr::Const(_) => Some(self.clone()),
            Expr::Pi(domain, codomain) => {
                let new_codomain =
                    codomain.substitute(target.checked_add(1)?, &replacement.shift(1, 0)?)?;
                Some(Expr::Pi(
                    Box::new(domain.substitute(target, replacement)?),
                    Box::new(new_codomain),
                ))
            }
            Expr::Lambda(parameter_type, body) => {
                let shifted_replacement = replacement.shift(1, 0)?;
                let new_body = body.substitute(target.checked_add(1)?, &shifted_replacement)?;
                Some(Expr::Lambda(
                    Box::new(parameter_type.substitute(target, replacement)?),
                    Box::new(new_body),
                ))
            }
            Expr::Application(func, arg) => Some(Expr::Application(
                Box::new(func.substitute(target, replacement)?),
                Box::new(arg.substitute(target, replacement)?),
            )),
        }
    }

    pub fn instantiate_pi(&self, argument: &Expr) -> Option<Expr> {
        match self {
            Expr::Pi(_, codomain) => {
                let shifted_argument = argument.shift(1, 0)?;
                let result = codomain.substitute(0, &shifted_argument)?;
                result.shift(-1, 0)
            }
            _ => None,
        }
    }

    pub fn apply_type(&self, argument: &Expr, argument_type: &Expr) -> Option<Expr> {
        let reduced_self = self.whnf()?;
        match &reduced_self {
            Expr::Pi(domain, _) => {
                if domain.def_eq(argument_type)? {
                    reduced_self.instantiate_pi(argument)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn infer_sort(&self) -> Option<Expr> {
        match self {
            Expr::Sort(u) => Some(Expr::Sort(u.checked_add(1)?)),
            _ => None,
        }
    }

    pub fn infer_type_with_env(
        &self,
        environment: &[Declaration],
        context: &[Expr],
    ) -> Option<Expr> {
        fn imax(u: usize, v: usize) -> usize {
            if v == 0 { v } else { max(u, v) }
        }

        match self {
            Expr::Sort(u) => Some(Expr::Sort(u.checked_add(1)?)),
            Expr::Index(index) => lookup_expr_type(context, *index),
            Expr::Const(index) => lookup_constant_type(environment, *index),
            Expr::Application(func, arg) => {
                let func_type = func.infer_type_with_env(environment, context)?;
                let arg_type = arg.infer_type_with_env(environment, context)?;

                func_type.apply_type(arg, &arg_type)
            }
            Expr::Lambda(parameter_type, body) => {
                if let Expr::Sort(_) = parameter_type
                    .infer_type_with_env(environment, context)?
                    .whnf()?
                {
                    let mut temp_context = Vec::with_capacity(context.len() + 1);
                    temp_context.push(*parameter_type.clone());
                    temp_context.extend_from_slice(context);

                    let body_type = body.infer_type_with_env(environment, &temp_context)?;
                    Some(Expr::Pi(parameter_type.clone(), Box::new(body_type)))
                } else {
                    None
                }
            }
            Expr::Pi(domain, codomain) => {
                let domain_type = domain.infer_type_with_env(environment, context)?;

                if let Expr::Sort(u) = domain_type.whnf()? {
                    let mut temp_context = Vec::with_capacity(context.len() + 1);
                    temp_context.push(*domain.clone());
                    temp_context.extend_from_slice(context);

                    if let Expr::Sort(v) = codomain
                        .infer_type_with_env(environment, &temp_context)?
                        .whnf()?
                    {
                        let result_level = imax(u, v);
                        return Some(Expr::Sort(result_level));
                    }
                    return None;
                }
                None
            }
        }
    }

    pub fn infer_type(&self, context: &[Expr]) -> Option<Expr> {
        self.infer_type_with_env(&[], context)
    }

    pub fn instantiate_lambda(&self, argument: &Expr) -> Option<Expr> {
        match self {
            Expr::Lambda(_, body) => {
                let shifted_argument = argument.shift(1, 0)?;
                let result = body.substitute(0, &shifted_argument)?;
                result.shift(-1, 0)
            }
            _ => None,
        }
    }

    // WHNF - Weak-Head Normal Form
    pub fn whnf(&self) -> Option<Expr> {
        match self {
            Expr::Application(func, arg) => {
                let func_whnf = func.whnf()?;

                if let Expr::Lambda(_, _) = func_whnf {
                    func_whnf.instantiate_lambda(arg)?.whnf()
                } else {
                    Some(Expr::Application(Box::new(func_whnf), arg.clone()))
                }
            }
            _ => Some(self.clone()),
        }
    }

    pub fn def_eq(&self, other: &Expr) -> Option<bool> {
        if self == other {
            return Some(true);
        }

        let lhs = self.whnf()?;
        let rhs = other.whnf()?;

        match (lhs, rhs) {
            (Expr::Sort(u), Expr::Sort(v)) => Some(u == v),
            (Expr::Index(i), Expr::Index(j)) => Some(i == j),
            (Expr::Const(i), Expr::Const(j)) => Some(i == j),
            (Expr::Pi(a1, b1), Expr::Pi(a2, b2)) => Some(a1.def_eq(&a2)? && b1.def_eq(&b2)?),
            (Expr::Lambda(type1, body1), Expr::Lambda(type2, body2)) => {
                Some(type1.def_eq(&type2)? && body1.def_eq(&body2)?)
            }
            (Expr::Application(f1, a1), Expr::Application(f2, a2)) => {
                Some(f1.def_eq(&f2)? && a1.def_eq(&a2)?)
            }
            _ => Some(false),
        }
    }

    pub fn check_type_with_env(
        &self,
        expected: &Expr,
        environment: &[Declaration],
        context: &[Expr],
    ) -> Option<bool> {
        if !environment_well_formed(environment) {
            return Some(false);
        }

        if !context_well_formed_with_env(environment, context) {
            return Some(false);
        }

        let actual = self.infer_type_with_env(environment, context)?;
        actual.def_eq(expected)
    }

    pub fn check_type(&self, expected: &Expr, context: &[Expr]) -> Option<bool> {
        self.check_type_with_env(expected, &[], context)
    }
}

pub fn lookup_expr_type(context: &[Expr], index: usize) -> Option<Expr> {
    let stored = context.get(index)?;
    let distance = isize::try_from(index.checked_add(1)?).ok()?;

    stored.shift(distance, 0)
}

pub fn context_well_formed_with_env(environment: &[Declaration], context: &[Expr]) -> bool {
    if context.is_empty() {
        return true;
    }
    let Some((first, rest)) = context.split_first() else {
        return false;
    };

    if !context_well_formed_with_env(environment, rest) {
        return false;
    }

    let Some(infered_type) = first.infer_type_with_env(environment, rest) else {
        return false;
    };

    matches!(infered_type.whnf(), Some(Expr::Sort(_)))
}

pub fn context_well_formed(context: &[Expr]) -> bool {
    context_well_formed_with_env(&[], context)
}

pub fn lookup_constant_type(environment: &[Declaration], index: usize) -> Option<Expr> {
    let const_var = environment.get(index)?;
    Some(const_var.ty.clone())
}

pub fn environment_well_formed(environment: &[Declaration]) -> bool {
    if environment.is_empty() {
        return true;
    }
    for index in 0..environment.len() {
        let Some(dec) = environment.get(index) else {
            return false;
        };

        let Some(prior_environment) = environment.get(..index) else {
            return false;
        };

        let Some(dec_type) = dec.ty.infer_type_with_env(prior_environment, &[]) else {
            return false;
        };

        if !matches!(dec_type.whnf(), Some(Expr::Sort(_))) {
            return false;
        };

        if let Some(dec_value) = &dec.value {
            let Some(dec_value_type) = dec_value.infer_type_with_env(prior_environment, &[]) else {
                return false;
            };

            if Some(true) != dec_value_type.def_eq(&dec.ty) {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::Declaration;
    use super::Expr;
    use super::Term::*;
    use super::Type::*;
    use super::context_well_formed;
    use super::environment_well_formed;
    use super::lookup_constant_type;
    use super::lookup_expr_type;
    use super::lookup_type;

    // SHIFT //
    #[test]
    fn test_shift_single_lambda() {
        let term = Lambda(
            Base,
            Box::new(Application(Box::new(Index(1)), Box::new(Index(0)))),
        );
        let result = Lambda(
            Base,
            Box::new(Application(Box::new(Index(2)), Box::new(Index(0)))),
        );

        assert_eq!(term.shift(1, 0), Some(result))
    }

    #[test]
    fn test_shift_multi_lambda_nested_app() {
        let term = Lambda(
            Base,
            Box::new(Lambda(
                Base,
                Box::new(Application(
                    Box::new(Application(Box::new(Index(2)), Box::new(Index(1)))),
                    Box::new(Index(0)),
                )),
            )),
        );
        let result = Lambda(
            Base,
            Box::new(Lambda(
                Base,
                Box::new(Application(
                    Box::new(Application(Box::new(Index(3)), Box::new(Index(1)))),
                    Box::new(Index(0)),
                )),
            )),
        );

        assert_eq!(term.shift(1, 0), Some(result))
    }

    #[test]
    fn test_shift_no_lambda() {
        let term = Application(Box::new(Index(1)), Box::new(Index(0)));
        let result = Application(Box::new(Index(3)), Box::new(Index(2)));

        assert_eq!(term.shift(2, 0), Some(result))
    }

    #[test]
    fn test_shift_no_lambda_2() {
        let term = Application(Box::new(Index(2)), Box::new(Index(1)));
        let result = Application(Box::new(Index(3)), Box::new(Index(1)));

        assert_eq!(term.shift(1, 2), Some(result))
    }

    #[test]
    fn test_shift_valid_downshift() {
        let term = Application(Box::new(Index(3)), Box::new(Index(2)));
        let result = Application(Box::new(Index(1)), Box::new(Index(0)));

        assert_eq!(term.shift(-2, 0), Some(result))
    }

    #[test]
    fn test_shift_invalid_downshift() {
        let term = Index(0);
        let result = None;

        assert_eq!(term.shift(-1, 0), result)
    }

    #[test]
    fn test_shift_valid_downshift_with_cutoff() {
        let term = Lambda(
            Base,
            Box::new(Application(Box::new(Index(2)), Box::new(Index(0)))),
        );
        let result = Some(Lambda(
            Base,
            Box::new(Application(Box::new(Index(1)), Box::new(Index(0)))),
        ));

        assert_eq!(term.shift(-1, 0), result)
    }

    // SUBSTITUTE //
    #[test]
    fn test_substitue_simple_index() {
        let term = Index(0);
        let replacement = Index(5);
        let result = Index(5);
        assert_eq!(term.substitute(0, &replacement), Some(result))
    }

    #[test]
    fn test_substitue_simple_app() {
        let term = Application(Box::new(Index(0)), Box::new(Index(1)));
        let replacement = Index(5);
        let result = Application(Box::new(Index(5)), Box::new(Index(1)));
        assert_eq!(term.substitute(0, &replacement), Some(result))
    }

    #[test]
    fn test_substitue_simple_binder() {
        let term = Lambda(Base, Box::new(Index(1)));
        let replacement = Index(5);
        let result = Lambda(Base, Box::new(Index(6)));
        assert_eq!(term.substitute(0, &replacement), Some(result))
    }

    #[test]
    fn test_substitue_shadow_casing() {
        let term = Lambda(Base, Box::new(Index(0)));
        let replacement = Index(5);
        let result = Lambda(Base, Box::new(Index(0)));
        assert_eq!(term.substitute(0, &replacement), Some(result))
    }

    #[test]
    fn test_substitue_duplication() {
        let term = Application(Box::new(Index(0)), Box::new(Index(0)));
        let replacement = Index(5);
        let result = Application(Box::new(Index(5)), Box::new(Index(5)));
        assert_eq!(term.substitute(0, &replacement), Some(result))
    }

    #[test]
    fn test_substitue_cum_binder_depth() {
        let term = Lambda(Base, Box::new(Lambda(Base, Box::new(Index(2)))));
        let replacement = Index(5);
        let result = Lambda(Base, Box::new(Lambda(Base, Box::new(Index(7)))));
        assert_eq!(term.substitute(0, &replacement), Some(result))
    }

    // BETA-REDUCE
    #[test]
    fn test_beta_reduce_identity() {
        let term = Application(
            Box::new(Lambda(Base, Box::new(Index(0)))),
            Box::new(Lambda(Base, Box::new(Index(0)))),
        );
        let result = Lambda(Base, Box::new(Index(0)));

        assert_eq!(term.beta_reduce(), Some(result))
    }

    #[test]
    fn test_beta_reduce_constant_function() {
        let term = Application(
            Box::new(Lambda(Base, Box::new(Lambda(Base, Box::new(Index(1)))))),
            Box::new(Index(0)),
        );
        let result = Lambda(Base, Box::new(Index(1)));

        assert_eq!(term.beta_reduce(), Some(result))
    }

    #[test]
    fn test_beta_reduce_arg_twice() {
        let term = Application(
            Box::new(Lambda(
                Base,
                Box::new(Application(Box::new(Index(0)), Box::new(Index(0)))),
            )),
            Box::new(Lambda(Base, Box::new(Index(0)))),
        );
        let result = Application(
            Box::new(Lambda(Base, Box::new(Index(0)))),
            Box::new(Lambda(Base, Box::new(Index(0)))),
        );

        assert_eq!(term.beta_reduce(), Some(result))
    }

    #[test]
    fn test_beta_reduce_no_redex() {
        let term = Application(Box::new(Index(0)), Box::new(Index(1)));
        let result = None;

        assert_eq!(term.beta_reduce(), result)
    }

    // TYPE
    #[test]
    fn base_type_equals_itself() {
        assert_eq!(Base, Base);
    }

    #[test]
    fn arrow_type_has_expected_structure() {
        let actual = Arrow(Box::new(Base), Box::new(Base));

        let expected = Arrow(Box::new(Base), Box::new(Base));

        assert_eq!(actual, expected);
    }

    #[test]
    fn typed_identity_has_two_term_nodes() {
        let term = Lambda(Base, Box::new(Index(0)));

        assert_eq!(term.count_nodes(), 2);
    }

    #[test]
    fn test_shift_preserves_lambda_parameter_type() {
        let parameter_type = Arrow(Box::new(Base), Box::new(Base));

        let term = Lambda(parameter_type.clone(), Box::new(Index(1)));

        let expected = Lambda(parameter_type, Box::new(Index(2)));

        assert_eq!(term.shift(1, 0), Some(expected));
    }

    // LookUp Type
    #[test]
    fn lookup_nearest_binder() {
        let context = vec![Arrow(Box::new(Base), Box::new(Base)), Base];

        assert_eq!(
            lookup_type(&context, 0),
            Some(Arrow(Box::new(Base), Box::new(Base)))
        );
    }

    #[test]
    fn lookup_outer_binder() {
        let context = vec![Arrow(Box::new(Base), Box::new(Base)), Base];

        assert_eq!(lookup_type(&context, 1), Some(Base));
    }

    #[test]
    fn lookup_missing_binder() {
        let context = vec![Base];

        assert_eq!(lookup_type(&context, 1), None);
    }

    // Infer Types
    #[test]
    fn infer_index_type() {
        let context = vec![Base];
        let term = Index(0);

        assert_eq!(term.infer_type(&context), Some(Base));
    }

    #[test]
    fn infer_unbound_index_fails() {
        let context = vec![];
        let term = Index(0);

        assert_eq!(term.infer_type(&context), None);
    }

    #[test]
    fn infer_identity_type() {
        let term = Lambda(Base, Box::new(Index(0)));

        let expected = Arrow(Box::new(Base), Box::new(Base));

        assert_eq!(term.infer_type(&[]), Some(expected));
    }

    #[test]
    fn infer_lambda_uses_new_binder_as_index_zero() {
        let outer_context = vec![Arrow(Box::new(Base), Box::new(Base))];

        let term = Lambda(Base, Box::new(Index(0)));

        let expected = Arrow(Box::new(Base), Box::new(Base));

        assert_eq!(term.infer_type(&outer_context), Some(expected));
    }

    #[test]
    fn infer_constant_function_type() {
        let term = Lambda(Base, Box::new(Lambda(Base, Box::new(Index(1)))));

        let expected = Arrow(
            Box::new(Base),
            Box::new(Arrow(Box::new(Base), Box::new(Base))),
        );

        assert_eq!(term.infer_type(&[]), Some(expected));
    }

    #[test]
    fn infer_valid_application() {
        let context = vec![Arrow(Box::new(Base), Box::new(Base)), Base];

        let term = Application(Box::new(Index(0)), Box::new(Index(1)));

        assert_eq!(term.infer_type(&context), Some(Base));
    }

    #[test]
    fn infer_application_wrong_argument_type_fails() {
        let context = vec![
            Arrow(Box::new(Base), Box::new(Base)),
            Arrow(Box::new(Base), Box::new(Base)),
        ];

        let term = Application(Box::new(Index(0)), Box::new(Index(1)));

        assert_eq!(term.infer_type(&context), None);
    }

    #[test]
    fn infer_application_non_function_fails() {
        let context = vec![Base, Base];

        let term = Application(Box::new(Index(0)), Box::new(Index(1)));

        assert_eq!(term.infer_type(&context), None);
    }

    #[test]
    fn infer_lambda_can_reference_outer_context() {
        let context = vec![Arrow(Box::new(Base), Box::new(Base))];

        let term = Lambda(Base, Box::new(Index(1)));

        let expected = Arrow(
            Box::new(Base),
            Box::new(Arrow(Box::new(Base), Box::new(Base))),
        );

        assert_eq!(term.infer_type(&context), Some(expected));
    }

    #[test]
    fn infer_nested_lambda_outer_binder() {
        let term = Lambda(
            Base,
            Box::new(Lambda(
                Arrow(Box::new(Base), Box::new(Base)),
                Box::new(Index(1)),
            )),
        );

        let expected = Arrow(
            Box::new(Base),
            Box::new(Arrow(
                Box::new(Arrow(Box::new(Base), Box::new(Base))),
                Box::new(Base),
            )),
        );

        assert_eq!(term.infer_type(&[]), Some(expected));
    }

    #[test]
    fn infer_identity_application() {
        let term = Application(
            Box::new(Lambda(Base, Box::new(Index(0)))),
            Box::new(Index(0)),
        );

        let context = vec![Base];

        assert_eq!(term.infer_type(&context), Some(Base));
    }

    // EXPR
    #[test]
    fn expr_prop_constructs() {
        let expr = Expr::Sort(0);

        assert_eq!(expr, Expr::Sort(0));
    }

    #[test]
    fn dependent_identity_type_has_correct_de_bruijn_indices() {
        // Π (A : Prop), Π (a : A), A
        //
        // Outer Pi:
        //   A : Prop
        //
        // Inner Pi domain:
        //   A = Index(0)
        //
        // Inner Pi codomain:
        //   Index(0) = a
        //   Index(1) = A
        let expr = Expr::Pi(
            Box::new(Expr::Sort(0)),
            Box::new(Expr::Pi(Box::new(Expr::Index(0)), Box::new(Expr::Index(1)))),
        );

        let expected = Expr::Pi(
            Box::new(Expr::Sort(0)),
            Box::new(Expr::Pi(Box::new(Expr::Index(0)), Box::new(Expr::Index(1)))),
        );

        assert_eq!(expr, expected);
    }

    #[test]
    fn pi_binder_is_not_in_scope_in_its_domain() {
        // Π (A : Prop), Π (_ : A), A
        //
        // When constructing the inner Pi:
        //
        // domain Index(0) refers to the already-existing outer A.
        //
        // In the codomain, the inner Pi has introduced a new binder,
        // so the same outer A must now be Index(1).
        let expr = Expr::Pi(
            Box::new(Expr::Sort(0)),
            Box::new(Expr::Pi(Box::new(Expr::Index(0)), Box::new(Expr::Index(1)))),
        );

        match expr {
            Expr::Pi(_, outer_codomain) => match *outer_codomain {
                Expr::Pi(inner_domain, inner_codomain) => {
                    assert_eq!(*inner_domain, Expr::Index(0));
                    assert_eq!(*inner_codomain, Expr::Index(1));
                }
                other => panic!("expected inner Pi, got: {:?}", other),
            },
            other => panic!("expected outer Pi, got: {:?}", other),
        }
    }

    #[test]
    fn lambda_binder_is_only_in_scope_in_body() {
        // Inside an outer A : Prop context, represent:
        //
        // fun (a : A) => a
        //
        // parameter type:
        //   Index(0) = outer A
        //
        // body:
        //   after Lambda introduces `a`,
        //   Index(0) = a
        let expr = Expr::Lambda(Box::new(Expr::Index(0)), Box::new(Expr::Index(0)));

        match expr {
            Expr::Lambda(parameter_type, body) => {
                assert_eq!(*parameter_type, Expr::Index(0));
                assert_eq!(*body, Expr::Index(0));
            }
            other => panic!("expected Lambda, got: {:?}", other),
        }
    }

    #[test]
    fn application_introduces_no_binder() {
        // f a
        //
        // Application itself does not alter de Bruijn depth.
        let expr = Expr::Application(Box::new(Expr::Index(1)), Box::new(Expr::Index(0)));

        match expr {
            Expr::Application(function, argument) => {
                assert_eq!(*function, Expr::Index(1));
                assert_eq!(*argument, Expr::Index(0));
            }
            other => panic!("expected Application, got: {:?}", other),
        }
    }

    // EXPR Shift Test
    #[test]
    fn expr_shift_prop_is_unchanged() {
        let expr = Expr::Sort(0);

        assert_eq!(expr.shift(1, 0), Some(Expr::Sort(0)));
    }

    #[test]
    fn expr_shift_free_index_up() {
        let expr = Expr::Index(0);

        assert_eq!(expr.shift(1, 0), Some(Expr::Index(1)));
    }

    #[test]
    fn expr_shift_respects_cutoff() {
        let expr = Expr::Index(0);

        assert_eq!(expr.shift(1, 1), Some(Expr::Index(0)));
    }

    #[test]
    fn expr_shift_pi_domain_does_not_get_new_binder_cutoff() {
        // Imagine this Pi occurs inside a context where Index(0)
        // refers to some outer expression A:
        //
        // Π (_ : A), ...
        //
        // The Pi binder is NOT in scope in its domain.
        //
        // Therefore shifting the whole open expression at cutoff 0
        // must shift the domain Index(0) -> Index(1).
        let expr = Expr::Pi(Box::new(Expr::Index(0)), Box::new(Expr::Sort(0)));

        let expected = Expr::Pi(Box::new(Expr::Index(1)), Box::new(Expr::Sort(0)));

        assert_eq!(expr.shift(1, 0), Some(expected));
    }

    #[test]
    fn expr_shift_pi_codomain_respects_pi_binder() {
        // Π (_ : Prop), Index(0)
        //
        // Index(0) in the codomain refers to the Pi-bound variable,
        // so shifting the whole expression must NOT change it.
        let expr = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Index(0)));

        assert_eq!(expr.shift(1, 0), Some(expr));
    }

    #[test]
    fn expr_shift_lambda_parameter_type_does_not_see_lambda_binder() {
        // In an outer context:
        //
        // fun (_ : Index(0)) => ...
        //
        // The lambda binder is not in scope in its parameter type.
        let expr = Expr::Lambda(Box::new(Expr::Index(0)), Box::new(Expr::Sort(0)));

        let expected = Expr::Lambda(Box::new(Expr::Index(1)), Box::new(Expr::Sort(0)));

        assert_eq!(expr.shift(1, 0), Some(expected));
    }

    #[test]
    fn expr_shift_lambda_body_respects_lambda_binder() {
        let expr = Expr::Lambda(Box::new(Expr::Sort(0)), Box::new(Expr::Index(0)));

        assert_eq!(expr.shift(1, 0), Some(expr));
    }

    #[test]
    fn expr_shift_closed_dependent_identity_type_is_unchanged() {
        // Π (A : Prop), Π (a : A), A
        let expr = Expr::Pi(
            Box::new(Expr::Sort(0)),
            Box::new(Expr::Pi(Box::new(Expr::Index(0)), Box::new(Expr::Index(1)))),
        );

        assert_eq!(expr.shift(1, 0), Some(expr));
    }

    #[test]
    fn expr_shift_application_recurses_into_both_children() {
        let expr = Expr::Application(Box::new(Expr::Index(0)), Box::new(Expr::Index(2)));

        let expected = Expr::Application(Box::new(Expr::Index(1)), Box::new(Expr::Index(3)));

        assert_eq!(expr.shift(1, 0), Some(expected));
    }

    #[test]
    fn expr_shift_invalid_downward_shift_returns_none() {
        let expr = Expr::Index(0);

        assert_eq!(expr.shift(-1, 0), None);
    }

    // Expr Substitute
    #[test]
    fn expr_substitute_prop_is_unchanged() {
        let expr = Expr::Sort(0);
        let replacement = Expr::Index(4);

        assert_eq!(expr.substitute(0, &replacement), Some(Expr::Sort(0)));
    }

    #[test]
    fn expr_substitute_matching_index() {
        let expr = Expr::Index(0);
        let replacement = Expr::Sort(0);

        assert_eq!(expr.substitute(0, &replacement), Some(Expr::Sort(0)));
    }

    #[test]
    fn expr_substitute_nonmatching_index() {
        let expr = Expr::Index(1);
        let replacement = Expr::Sort(0);

        assert_eq!(expr.substitute(0, &replacement), Some(Expr::Index(1)));
    }

    #[test]
    fn expr_substitute_application_recurses_into_both_children() {
        let expr = Expr::Application(Box::new(Expr::Index(0)), Box::new(Expr::Index(0)));

        let replacement = Expr::Sort(0);

        let expected = Expr::Application(Box::new(Expr::Sort(0)), Box::new(Expr::Sort(0)));

        assert_eq!(expr.substitute(0, &replacement), Some(expected));
    }

    #[test]
    fn expr_substitute_pi_domain_without_crossing_pi_binder() {
        // The Pi binder does not exist in its domain.
        let expr = Expr::Pi(Box::new(Expr::Index(0)), Box::new(Expr::Sort(0)));

        let replacement = Expr::Sort(0);

        let expected = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Sort(0)));

        assert_eq!(expr.substitute(0, &replacement), Some(expected));
    }

    #[test]
    fn expr_substitute_pi_codomain_crosses_binder() {
        // We substitute outer target 0.
        //
        // Inside the Pi codomain:
        // target becomes 1.
        //
        // Index(0) is the Pi-bound variable and must remain untouched.
        // Index(1) refers to the outer target and must be replaced.
        let expr = Expr::Pi(
            Box::new(Expr::Sort(0)),
            Box::new(Expr::Application(
                Box::new(Expr::Index(1)),
                Box::new(Expr::Index(0)),
            )),
        );

        let replacement = Expr::Sort(0);

        let expected = Expr::Pi(
            Box::new(Expr::Sort(0)),
            Box::new(Expr::Application(
                Box::new(Expr::Sort(0)),
                Box::new(Expr::Index(0)),
            )),
        );

        assert_eq!(expr.substitute(0, &replacement), Some(expected));
    }

    #[test]
    fn expr_substitute_lambda_parameter_type_without_crossing_binder() {
        let expr = Expr::Lambda(Box::new(Expr::Index(0)), Box::new(Expr::Sort(0)));

        let replacement = Expr::Sort(0);

        let expected = Expr::Lambda(Box::new(Expr::Sort(0)), Box::new(Expr::Sort(0)));

        assert_eq!(expr.substitute(0, &replacement), Some(expected));
    }

    #[test]
    fn expr_substitute_lambda_body_crosses_binder() {
        let expr = Expr::Lambda(
            Box::new(Expr::Sort(0)),
            Box::new(Expr::Application(
                Box::new(Expr::Index(1)),
                Box::new(Expr::Index(0)),
            )),
        );

        let replacement = Expr::Sort(0);

        let expected = Expr::Lambda(
            Box::new(Expr::Sort(0)),
            Box::new(Expr::Application(
                Box::new(Expr::Sort(0)),
                Box::new(Expr::Index(0)),
            )),
        );

        assert_eq!(expr.substitute(0, &replacement), Some(expected));
    }

    #[test]
    fn expr_substitute_shifts_replacement_when_crossing_pi_binder() {
        // Replacement Index(0) refers to something in the caller's context.
        //
        // When inserted under the Pi binder it must become Index(1)
        // so that it is not captured by the new binder.
        let expr = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Index(1)));

        let replacement = Expr::Index(0);

        let expected = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Index(1)));

        assert_eq!(expr.substitute(0, &replacement), Some(expected));
    }

    #[test]
    fn expr_substitute_shifts_replacement_through_multiple_binders() {
        // Outer target appears as Index(2) under two nested Pi binders.
        // Replacement Index(0) must be shifted twice -> Index(2).
        let expr = Expr::Pi(
            Box::new(Expr::Sort(0)),
            Box::new(Expr::Pi(Box::new(Expr::Index(0)), Box::new(Expr::Index(2)))),
        );

        let replacement = Expr::Index(0);

        let expected = Expr::Pi(
            Box::new(Expr::Sort(0)),
            Box::new(Expr::Pi(Box::new(Expr::Index(0)), Box::new(Expr::Index(2)))),
        );

        assert_eq!(expr.substitute(0, &replacement), Some(expected));
    }

    // Intantiate PI
    #[test]
    fn expr_instantiate_pi_constant_codomain() {
        // Π (_ : Prop), Prop
        //
        // The codomain does not depend on the Pi-bound variable,
        // so instantiating with any argument leaves it unchanged.
        let pi = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Sort(0)));

        let argument = Expr::Index(0);

        assert_eq!(pi.instantiate_pi(&argument), Some(Expr::Sort(0)));
    }

    #[test]
    fn expr_instantiate_pi_direct_dependent_codomain() {
        // Π (x : Prop), x
        //
        // Codomain Index(0) is the Pi-bound x.
        // Instantiating x := Prop should therefore produce Prop.
        let pi = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Index(0)));

        let argument = Expr::Sort(0);

        assert_eq!(pi.instantiate_pi(&argument), Some(Expr::Sort(0)));
    }

    #[test]
    fn expr_instantiate_pi_removes_binder_from_outer_reference() {
        // Imagine an outer context containing A at Index(0).
        //
        // Π (x : A), A
        //
        // Pi domain:
        //   Index(0) = outer A
        //
        // Pi codomain:
        //   Index(0) = x
        //   Index(1) = outer A
        //
        // After instantiation, the Pi binder disappears,
        // so outer A must return to Index(0).
        let pi = Expr::Pi(Box::new(Expr::Index(0)), Box::new(Expr::Index(1)));

        let argument = Expr::Sort(0);

        assert_eq!(pi.instantiate_pi(&argument), Some(Expr::Index(0)));
    }

    #[test]
    fn expr_instantiate_pi_avoids_capture_under_nested_lambda() {
        // Imagine argument Index(0) refers to something in the
        // caller's outer context.
        //
        // Pi:
        //
        // Π (x : Prop),
        //   fun (_ : Prop) => x
        //
        // Inside the Lambda body:
        //
        //   Index(0) = lambda-bound variable
        //   Index(1) = Pi-bound x
        //
        // We instantiate x with caller's Index(0).
        //
        // After the Pi binder disappears, the result should be:
        //
        //   fun (_ : Prop) => Index(1)
        //
        // NOT Index(0), because that would capture the argument
        // with the nested Lambda binder.
        let pi = Expr::Pi(
            Box::new(Expr::Sort(0)),
            Box::new(Expr::Lambda(
                Box::new(Expr::Sort(0)),
                Box::new(Expr::Index(1)),
            )),
        );

        let argument = Expr::Index(0);

        let expected = Expr::Lambda(Box::new(Expr::Sort(0)), Box::new(Expr::Index(1)));

        assert_eq!(pi.instantiate_pi(&argument), Some(expected));
    }

    #[test]
    fn expr_instantiate_pi_returns_none_for_non_pi() {
        let expr = Expr::Application(Box::new(Expr::Index(1)), Box::new(Expr::Index(0)));

        let argument = Expr::Sort(0);

        assert_eq!(expr.instantiate_pi(&argument), None);
    }

    #[test]
    fn expr_apply_type_rejects_non_pi_function_type() {
        let function_type = Expr::Sort(0);
        let argument = Expr::Sort(0);
        let argument_type = Expr::Sort(0);

        assert_eq!(function_type.apply_type(&argument, &argument_type), None);
    }

    #[test]
    fn expr_apply_type_rejects_wrong_argument_type() {
        // Function type:
        //
        // Π (_ : Prop), Prop
        let function_type = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Sort(0)));

        let argument = Expr::Index(0);

        // Deliberately does not match domain Prop.
        let argument_type = Expr::Index(0);

        assert_eq!(function_type.apply_type(&argument, &argument_type), None);
    }

    #[test]
    fn expr_apply_type_non_dependent_pi_returns_fixed_codomain() {
        // Π (_ : Prop), Prop
        let function_type = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Sort(0)));

        let argument = Expr::Sort(0);
        let argument_type = Expr::Sort(0);

        assert_eq!(
            function_type.apply_type(&argument, &argument_type),
            Some(Expr::Sort(0))
        );
    }

    #[test]
    fn expr_apply_type_dependent_codomain_is_instantiated() {
        // Function type:
        //
        // Π (x : Prop), x
        //
        // Again, we're testing the dependent application mechanism,
        // not whether this is a meaningful Lean declaration.
        let function_type = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Index(0)));

        let argument = Expr::Sort(0);
        let argument_type = Expr::Sort(0);

        assert_eq!(
            function_type.apply_type(&argument, &argument_type),
            Some(Expr::Sort(0))
        );
    }

    #[test]
    fn expr_apply_type_removes_pi_binder_from_outer_reference() {
        // Imagine outer context:
        //
        // A : ?
        //
        // Function type:
        //
        // Π (x : A), A
        //
        // domain    = Index(0)  -- outer A
        // codomain  = Index(1)  -- outer A beneath x binder
        //
        // After application, the Pi binder disappears and
        // the result type should refer to outer A as Index(0).
        let function_type = Expr::Pi(Box::new(Expr::Index(0)), Box::new(Expr::Index(1)));

        let argument = Expr::Index(1);
        let argument_type = Expr::Index(0);

        assert_eq!(
            function_type.apply_type(&argument, &argument_type),
            Some(Expr::Index(0))
        );
    }

    // SORT
    #[test]
    fn expr_sort_levels_are_structurally_distinct() {
        assert_ne!(Expr::Sort(0), Expr::Sort(1));

        assert_ne!(Expr::Sort(1), Expr::Sort(2));
    }

    #[test]
    fn expr_shift_sort_preserves_level() {
        assert_eq!(Expr::Sort(0).shift(1, 0), Some(Expr::Sort(0)));

        assert_eq!(Expr::Sort(3).shift(-1, 0), Some(Expr::Sort(3)));
    }

    #[test]
    fn expr_substitute_sort_preserves_level() {
        let replacement = Expr::Index(5);

        assert_eq!(
            Expr::Sort(0).substitute(0, &replacement),
            Some(Expr::Sort(0))
        );

        assert_eq!(
            Expr::Sort(4).substitute(0, &replacement),
            Some(Expr::Sort(4))
        );
    }

    #[test]
    fn expr_infer_sort_zero() {
        assert_eq!(Expr::Sort(0).infer_sort(), Some(Expr::Sort(1)));
    }

    #[test]
    fn expr_infer_sort_higher_universe() {
        assert_eq!(Expr::Sort(4).infer_sort(), Some(Expr::Sort(5)));
    }

    #[test]
    fn expr_infer_sort_rejects_non_sort_for_now() {
        assert_eq!(Expr::Index(0).infer_sort(), None);
    }

    // LOOKUP EXPR TYPE
    #[test]
    fn lookup_expr_type_nearest_dependent_binder() {
        // Context represents:
        //
        // A : Sort(0)
        // a : A
        //
        // nearest first:
        //
        // context[0] = type of a, stored as A = Index(0)
        // context[1] = type of A = Sort(0)
        let context = vec![Expr::Index(0), Expr::Sort(0)];

        // In the current context:
        //
        // Index(0) = a
        // Index(1) = A
        //
        // Therefore:
        //
        // a : Index(1)
        assert_eq!(lookup_expr_type(&context, 0), Some(Expr::Index(1)));
    }

    #[test]
    fn lookup_expr_type_outer_sort() {
        let context = vec![Expr::Index(0), Expr::Sort(0)];

        // Index(1) is A.
        // A : Sort(0).
        //
        // Sort is unaffected by shifting.
        assert_eq!(lookup_expr_type(&context, 1), Some(Expr::Sort(0)));
    }

    #[test]
    fn lookup_expr_type_shifts_through_multiple_newer_binders() {
        // Imagine an outer type referring to something still farther out.
        //
        // context[2] was stored as Index(0), but by the time we
        // look it up there are 3 binders between its original
        // environment and the current one.
        let context = vec![Expr::Sort(0), Expr::Sort(0), Expr::Index(0)];

        assert_eq!(lookup_expr_type(&context, 2), Some(Expr::Index(3)));
    }

    #[test]
    fn lookup_expr_type_missing_index_returns_none() {
        let context = vec![Expr::Sort(0)];

        assert_eq!(lookup_expr_type(&context, 1), None);
    }

    // INFER TYPE - EXPR
    // Sorts
    #[test]
    fn expr_infer_type_sort() {
        assert_eq!(Expr::Sort(0).infer_type(&[]), Some(Expr::Sort(1)));

        assert_eq!(Expr::Sort(3).infer_type(&[]), Some(Expr::Sort(4)));
    }

    // Variables
    #[test]
    fn expr_infer_type_dependent_variable() {
        // A : Prop
        // a : A
        //
        // Current context:
        // Index(0) = a
        // Index(1) = A
        let context = vec![Expr::Index(0), Expr::Sort(0)];

        assert_eq!(Expr::Index(0).infer_type(&context), Some(Expr::Index(1)));

        assert_eq!(Expr::Index(1).infer_type(&context), Some(Expr::Sort(0)));
    }

    // Unbounded Variables
    #[test]
    fn expr_infer_type_rejects_unbound_index() {
        assert_eq!(Expr::Index(0).infer_type(&[]), None);
    }

    // Indentity Lambda over Prop
    #[test]
    fn expr_infer_type_identity_lambda_over_prop() {
        // fun (A : Prop) => A
        //
        // This is not yet our full polymorphic identity proof;
        // the body simply returns the proposition A itself.
        let expr = Expr::Lambda(Box::new(Expr::Sort(0)), Box::new(Expr::Index(0)));

        let expected = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Sort(0)));

        assert_eq!(expr.infer_type(&[]), Some(expected));
    }

    // Polymorphic Indentity Proof
    #[test]
    fn expr_infer_type_polymorphic_identity() {
        // fun (A : Prop) =>
        //   fun (a : A) =>
        //     a

        let expr = Expr::Lambda(
            Box::new(Expr::Sort(0)),
            Box::new(Expr::Lambda(
                Box::new(Expr::Index(0)),
                Box::new(Expr::Index(0)),
            )),
        );

        let expected = Expr::Pi(
            Box::new(Expr::Sort(0)),
            Box::new(Expr::Pi(Box::new(Expr::Index(0)), Box::new(Expr::Index(1)))),
        );

        assert_eq!(expr.infer_type(&[]), Some(expected));
    }

    // PI returning Prop stays Prop
    #[test]
    fn expr_infer_type_pi_into_prop_is_prop() {
        // Π (A : Prop), A
        //
        // domain:
        //   Prop : Sort(1)
        //
        // codomain:
        //   A : Prop = Sort(0)
        //
        // Because codomain sort is 0, imax(1, 0) = 0.
        let expr = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Index(0)));

        assert_eq!(expr.infer_type(&[]), Some(Expr::Sort(0)));
    }

    // Ordinary Universe Max
    #[test]
    fn expr_infer_type_pi_uses_max_when_codomain_not_prop() {
        // Π (_ : Sort(1)), Sort(2)
        //
        // domain      Sort(1) : Sort(2)  => u = 2
        // codomain    Sort(2) : Sort(3)  => v = 3
        //
        // result = Sort(max(2, 3)) = Sort(3)
        let expr = Expr::Pi(Box::new(Expr::Sort(1)), Box::new(Expr::Sort(2)));

        assert_eq!(expr.infer_type(&[]), Some(Expr::Sort(3)));
    }

    // Dependent Application
    #[test]
    fn expr_infer_type_dependent_application() {
        // Context conceptually:
        //
        // A : Prop
        // a : A
        // f : Π (_ : A), A
        //
        // nearest first.

        let context = vec![
            // f : A → A
            Expr::Pi(Box::new(Expr::Index(1)), Box::new(Expr::Index(2))),
            // a : A
            Expr::Index(0),
            // A : Prop
            Expr::Sort(0),
        ];

        let expr = Expr::Application(
            Box::new(Expr::Index(0)), // f
            Box::new(Expr::Index(1)), // a
        );

        assert_eq!(expr.infer_type(&context), Some(Expr::Index(2)));
    }

    #[test]
    fn expr_infer_type_rejects_term_whose_type_is_pi_as_lambda_parameter_type() {
        // This is a TERM:
        //
        //   fun (A : Prop) => A
        //
        // Its type is:
        //
        //   Π (A : Prop), Prop
        //
        // So the expression itself is not a type/sort.
        let term_not_type = Expr::Lambda(Box::new(Expr::Sort(0)), Box::new(Expr::Index(0)));

        // Now deliberately try to use that term itself as the
        // parameter type of another lambda:
        //
        //   fun (x : (fun (A : Prop) => A)) => x
        //
        // This must be rejected.
        let expr = Expr::Lambda(Box::new(term_not_type), Box::new(Expr::Index(0)));

        assert_eq!(expr.infer_type(&[]), None);
    }

    #[test]
    fn expr_infer_type_accepts_pi_as_lambda_parameter_type() {
        // A genuine function TYPE:
        //
        //   Π (_ : Prop), Prop
        let function_type = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Sort(0)));

        // fun (f : Π (_ : Prop), Prop) => f
        let expr = Expr::Lambda(Box::new(function_type.clone()), Box::new(Expr::Index(0)));

        let expected = Expr::Pi(Box::new(function_type.clone()), Box::new(function_type));

        assert_eq!(expr.infer_type(&[]), Some(expected));
    }

    #[test]
    fn expr_shift_returns_none_when_cutoff_cannot_cross_binder() {
        let expr = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Sort(0)));

        assert_eq!(expr.shift(1, usize::MAX), None);
    }

    #[test]
    fn expr_shift_lambda_returns_none_when_cutoff_cannot_cross_binder() {
        let expr = Expr::Lambda(Box::new(Expr::Sort(0)), Box::new(Expr::Index(0)));

        assert_eq!(expr.shift(1, usize::MAX), None);
    }

    #[test]
    fn expr_instantiate_lambda_identity() {
        let lambda = Expr::Lambda(Box::new(Expr::Sort(1)), Box::new(Expr::Index(0)));

        let argument = Expr::Sort(0);

        assert_eq!(lambda.instantiate_lambda(&argument), Some(Expr::Sort(0)));
    }

    #[test]
    fn expr_instantiate_lambda_avoids_capture() {
        // Outer Index(0) is supplied as argument.
        //
        // λ x. λ y. x
        //
        // Under the inner lambda:
        //   y = Index(0)
        //   x = Index(1)
        let lambda = Expr::Lambda(
            Box::new(Expr::Sort(0)),
            Box::new(Expr::Lambda(
                Box::new(Expr::Sort(0)),
                Box::new(Expr::Index(1)),
            )),
        );

        let argument = Expr::Index(0);

        let expected = Expr::Lambda(Box::new(Expr::Sort(0)), Box::new(Expr::Index(1)));

        assert_eq!(lambda.instantiate_lambda(&argument), Some(expected));
    }

    #[test]
    fn expr_whnf_reduces_identity_application() {
        let expr = Expr::Application(
            Box::new(Expr::Lambda(
                Box::new(Expr::Sort(1)),
                Box::new(Expr::Index(0)),
            )),
            Box::new(Expr::Sort(0)),
        );

        assert_eq!(expr.whnf(), Some(Expr::Sort(0)));
    }

    #[test]
    fn expr_whnf_does_not_reduce_under_lambda() {
        let reducible_body = Expr::Application(
            Box::new(Expr::Lambda(
                Box::new(Expr::Sort(1)),
                Box::new(Expr::Index(0)),
            )),
            Box::new(Expr::Sort(0)),
        );

        let expr = Expr::Lambda(Box::new(Expr::Sort(0)), Box::new(reducible_body.clone()));

        assert_eq!(
            expr.whnf(),
            Some(Expr::Lambda(
                Box::new(Expr::Sort(0)),
                Box::new(reducible_body),
            ))
        );
    }

    #[test]
    fn expr_whnf_reduces_multiple_head_beta_steps() {
        // ((fun x => fun y => x) Sort(0)) Sort(1)
        //
        // → (fun y => Sort(0)) Sort(1)
        // → Sort(0)

        let expr = Expr::Application(
            Box::new(Expr::Application(
                Box::new(Expr::Lambda(
                    Box::new(Expr::Sort(1)),
                    Box::new(Expr::Lambda(
                        Box::new(Expr::Sort(2)),
                        Box::new(Expr::Index(1)),
                    )),
                )),
                Box::new(Expr::Sort(0)),
            )),
            Box::new(Expr::Sort(1)),
        );

        assert_eq!(expr.whnf(), Some(Expr::Sort(0)));
    }

    #[test]
    fn expr_whnf_does_not_reduce_application_argument() {
        let reducible_argument = Expr::Application(
            Box::new(Expr::Lambda(
                Box::new(Expr::Sort(1)),
                Box::new(Expr::Index(0)),
            )),
            Box::new(Expr::Sort(0)),
        );

        let expr = Expr::Application(
            Box::new(Expr::Index(0)),
            Box::new(reducible_argument.clone()),
        );

        assert_eq!(
            expr.whnf(),
            Some(Expr::Application(
                Box::new(Expr::Index(0)),
                Box::new(reducible_argument),
            ))
        );
    }

    #[test]
    fn expr_def_eq_structurally_equal() {
        let expr = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Index(0)));

        assert_eq!(expr.def_eq(&expr), Some(true));
    }

    #[test]
    fn expr_def_eq_distinguishes_sort_levels() {
        assert_eq!(Expr::Sort(0).def_eq(&Expr::Sort(1)), Some(false));
    }

    #[test]
    fn expr_def_eq_beta_reduces_left_side() {
        let left = Expr::Application(
            Box::new(Expr::Lambda(
                Box::new(Expr::Sort(1)),
                Box::new(Expr::Index(0)),
            )),
            Box::new(Expr::Sort(0)),
        );

        let right = Expr::Sort(0);

        assert_eq!(left.def_eq(&right), Some(true));
    }

    #[test]
    fn expr_def_eq_beta_reduces_right_side() {
        let left = Expr::Sort(0);

        let right = Expr::Application(
            Box::new(Expr::Lambda(
                Box::new(Expr::Sort(1)),
                Box::new(Expr::Index(0)),
            )),
            Box::new(Expr::Sort(0)),
        );

        assert_eq!(left.def_eq(&right), Some(true));
    }

    #[test]
    fn expr_def_eq_reduces_inside_pi_domain() {
        let reducible = Expr::Application(
            Box::new(Expr::Lambda(
                Box::new(Expr::Sort(1)),
                Box::new(Expr::Index(0)),
            )),
            Box::new(Expr::Sort(0)),
        );

        let left = Expr::Pi(Box::new(reducible), Box::new(Expr::Sort(0)));

        let right = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Sort(0)));

        assert_eq!(left.def_eq(&right), Some(true));
    }

    #[test]
    fn expr_def_eq_reduces_inside_pi_codomain() {
        let reducible = Expr::Application(
            Box::new(Expr::Lambda(
                Box::new(Expr::Sort(1)),
                Box::new(Expr::Index(0)),
            )),
            Box::new(Expr::Sort(0)),
        );

        let left = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(reducible));

        let right = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Sort(0)));

        assert_eq!(left.def_eq(&right), Some(true));
    }

    #[test]
    fn expr_def_eq_neutral_applications_compare_arguments() {
        let left = Expr::Application(Box::new(Expr::Index(0)), Box::new(Expr::Sort(0)));

        let right = Expr::Application(Box::new(Expr::Index(0)), Box::new(Expr::Sort(1)));

        assert_eq!(left.def_eq(&right), Some(false));
    }

    #[test]
    fn expr_def_eq_different_head_shapes_are_not_equal() {
        assert_eq!(Expr::Index(0).def_eq(&Expr::Sort(0)), Some(false));
    }

    #[test]
    fn expr_apply_type_accepts_definitionally_equal_argument_type() {
        // Function type:
        //
        // Π (_ : Sort(0)), Sort(0)
        let function_type = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Sort(0)));

        let argument = Expr::Sort(0);

        // This reduces to Sort(0):
        //
        // (fun (A : Sort(1)) => A) Sort(0)
        let argument_type = Expr::Application(
            Box::new(Expr::Lambda(
                Box::new(Expr::Sort(1)),
                Box::new(Expr::Index(0)),
            )),
            Box::new(Expr::Sort(0)),
        );

        assert_ne!(argument_type, Expr::Sort(0));

        assert_eq!(
            function_type.apply_type(&argument, &argument_type),
            Some(Expr::Sort(0))
        );
    }

    #[test]
    fn expr_apply_type_reduces_function_type_to_pi() {
        let pi_type = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Sort(0)));

        // A reducible expression whose WHNF is pi_type:
        //
        // (fun x => x) pi_type
        let reducible_function_type = Expr::Application(
            Box::new(Expr::Lambda(
                Box::new(Expr::Sort(2)),
                Box::new(Expr::Index(0)),
            )),
            Box::new(pi_type),
        );

        let argument = Expr::Sort(0);
        let argument_type = Expr::Sort(0);

        assert_eq!(
            reducible_function_type.apply_type(&argument, &argument_type),
            Some(Expr::Sort(0))
        );
    }

    #[test]
    fn expr_apply_type_still_rejects_definitionally_different_argument_type() {
        let function_type = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Sort(0)));

        let argument = Expr::Sort(0);
        let argument_type = Expr::Sort(1);

        assert_eq!(function_type.apply_type(&argument, &argument_type), None);
    }

    #[test]
    fn expr_infer_type_lambda_accepts_parameter_type_whose_type_reduces_to_sort() {
        // This expression reduces to Sort(0):
        //
        // (fun (X : Sort(1)) => X) Sort(0)
        let reducible_prop = Expr::Application(
            Box::new(Expr::Lambda(
                Box::new(Expr::Sort(1)),
                Box::new(Expr::Index(0)),
            )),
            Box::new(Expr::Sort(0)),
        );

        // Outer variable A has a type that is definitionally equal to Prop,
        // but is not syntactically Expr::Sort(0).
        let context = vec![reducible_prop];

        // fun (a : A) => a
        let expr = Expr::Lambda(Box::new(Expr::Index(0)), Box::new(Expr::Index(0)));

        let expected = Expr::Pi(Box::new(Expr::Index(0)), Box::new(Expr::Index(1)));

        assert_eq!(expr.infer_type(&context), Some(expected));
    }

    #[test]
    fn expr_infer_type_pi_accepts_domain_whose_type_reduces_to_sort() {
        // Definitionally equal to Sort(0), but not structurally Sort(0).
        let reducible_prop = Expr::Application(
            Box::new(Expr::Lambda(
                Box::new(Expr::Sort(1)),
                Box::new(Expr::Index(0)),
            )),
            Box::new(Expr::Sort(0)),
        );

        let context = vec![reducible_prop];

        // Π (_ : A), Prop
        //
        // A = Index(0)
        // A's type reduces to Sort(0), so A is a valid type.
        let expr = Expr::Pi(Box::new(Expr::Index(0)), Box::new(Expr::Sort(0)));

        assert_eq!(expr.infer_type(&context), Some(Expr::Sort(1)));
    }

    #[test]
    fn expr_infer_type_pi_accepts_codomain_whose_type_reduces_to_sort() {
        let reducible_prop = Expr::Application(
            Box::new(Expr::Lambda(
                Box::new(Expr::Sort(1)),
                Box::new(Expr::Index(0)),
            )),
            Box::new(Expr::Sort(0)),
        );

        // Outer A has type definitionally equal to Prop.
        let context = vec![reducible_prop];

        // Π (_ : Prop), A
        //
        // Inside the Pi codomain:
        // Index(0) = new binder
        // Index(1) = outer A
        let expr = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Index(1)));

        // Codomain A is itself a proposition, so the whole Pi is Prop.
        assert_eq!(expr.infer_type(&context), Some(Expr::Sort(0)));
    }

    #[test]
    fn expr_infer_type_must_not_beta_reduce_away_invalid_application() {
        // Function:
        //
        //   fun (_ : Prop) => Prop
        //
        // Its parameter type is Sort(0) = Prop, so an argument must
        // itself have type Prop / Sort(0).
        let func = Expr::Lambda(Box::new(Expr::Sort(0)), Box::new(Expr::Sort(0)));

        // Sort(1) has type Sort(2), NOT Sort(0).
        //
        // So this is an invalid argument for the function above.
        let bad_argument = Expr::Sort(1);

        let expr = Expr::Application(Box::new(func), Box::new(bad_argument));

        assert_eq!(expr.infer_type(&[]), None);
    }

    #[test]
    fn expr_check_type_accepts_polymorphic_identity() {
        // fun (A : Prop) =>
        //   fun (a : A) =>
        //     a
        let proof = Expr::Lambda(
            Box::new(Expr::Sort(0)),
            Box::new(Expr::Lambda(
                Box::new(Expr::Index(0)),
                Box::new(Expr::Index(0)),
            )),
        );

        // Π (A : Prop), Π (a : A), A
        let expected = Expr::Pi(
            Box::new(Expr::Sort(0)),
            Box::new(Expr::Pi(Box::new(Expr::Index(0)), Box::new(Expr::Index(1)))),
        );

        assert_eq!(proof.check_type(&expected, &[]), Some(true));
    }

    #[test]
    fn expr_check_type_rejects_wrong_claimed_type() {
        let proof = Expr::Lambda(
            Box::new(Expr::Sort(0)),
            Box::new(Expr::Lambda(
                Box::new(Expr::Index(0)),
                Box::new(Expr::Index(0)),
            )),
        );

        let wrong_expected = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Sort(0)));

        assert_eq!(proof.check_type(&wrong_expected, &[]), Some(false));
    }

    #[test]
    fn expr_check_type_accepts_definitionally_equal_expected_type() {
        // proof: fun (A : Prop) => A
        //
        // inferred type:
        // Π (_ : Prop), Prop
        let proof = Expr::Lambda(Box::new(Expr::Sort(0)), Box::new(Expr::Index(0)));

        // Reducible expression whose WHNF is Prop.
        let reducible_prop = Expr::Application(
            Box::new(Expr::Lambda(
                Box::new(Expr::Sort(1)),
                Box::new(Expr::Index(0)),
            )),
            Box::new(Expr::Sort(0)),
        );

        let expected = Expr::Pi(Box::new(Expr::Sort(0)), Box::new(reducible_prop));

        assert_eq!(proof.check_type(&expected, &[]), Some(true));
    }

    #[test]
    fn expr_check_type_propagates_inference_failure() {
        let unbound = Expr::Index(0);

        assert_eq!(unbound.check_type(&Expr::Sort(0), &[]), None);
    }

    #[test]
    fn empty_context_is_well_formed() {
        let context = vec![];

        assert!(context_well_formed(&context));
    }

    #[test]
    fn sort_context_is_well_formed() {
        let context = vec![Expr::Sort(0)];

        assert!(context_well_formed(&context));
    }

    #[test]
    fn dependent_context_is_well_formed() {
        let context = vec![Expr::Index(0), Expr::Sort(0)];

        assert!(context_well_formed(&context));
    }

    #[test]
    fn unbound_type_makes_context_malformed() {
        let context = vec![Expr::Index(0)];

        assert!(!context_well_formed(&context));
    }

    #[test]
    fn out_of_scope_type_makes_context_malformed() {
        let context = vec![Expr::Index(1), Expr::Sort(0)];

        assert!(!context_well_formed(&context));
    }

    #[test]
    fn well_typed_term_is_not_necessarily_a_valid_context_type() {
        let context = vec![Expr::Lambda(
            Box::new(Expr::Sort(0)),
            Box::new(Expr::Index(0)),
        )];

        assert!(!context_well_formed(&context));
    }

    #[test]
    fn pi_type_can_be_a_context_entry() {
        let context = vec![Expr::Pi(Box::new(Expr::Sort(0)), Box::new(Expr::Sort(0)))];

        assert!(context_well_formed(&context));
    }

    #[test]
    fn deeper_dependent_context_is_well_formed() {
        let context = vec![Expr::Index(1), Expr::Sort(0), Expr::Sort(0)];

        assert!(context_well_formed(&context));
    }

    #[test]
    fn check_type_rejects_malformed_context_even_for_closed_valid_term() {
        let context = vec![Expr::Index(0)];

        assert_eq!(
            Expr::Sort(0).check_type(&Expr::Sort(1), &context),
            Some(false)
        );
    }

    #[test]
    fn check_type_accepts_under_well_formed_context() {
        let context = vec![Expr::Sort(0)];

        assert_eq!(
            Expr::Sort(0).check_type(&Expr::Sort(1), &context),
            Some(true)
        );
    }

    #[test]
    fn constant_is_unaffected_by_shift() {
        assert_eq!(Expr::Const(0).shift(1, 0), Some(Expr::Const(0)));
    }

    #[test]
    fn constant_is_unaffected_by_substitution() {
        assert_eq!(
            Expr::Const(2).substitute(0, &Expr::Sort(0)),
            Some(Expr::Const(2))
        );
    }

    #[test]
    fn lookup_constant_type_works() {
        let env = vec![Declaration {
            ty: Expr::Sort(0),
            value: None,
        }];

        assert_eq!(lookup_constant_type(&env, 0), Some(Expr::Sort(0)));
    }

    #[test]
    fn constant_type_can_be_inferred() {
        let env = vec![Declaration {
            ty: Expr::Sort(0),
            value: None,
        }];

        assert_eq!(
            Expr::Const(0).infer_type_with_env(&env, &[]),
            Some(Expr::Sort(0))
        );
    }

    #[test]
    fn empty_environment_is_well_formed() {
        assert!(environment_well_formed(&[]));
    }

    #[test]
    fn simple_axiom_environment_is_well_formed() {
        let env = vec![Declaration {
            ty: Expr::Sort(0),
            value: None,
        }];

        assert!(environment_well_formed(&env));
    }

    #[test]
    fn declaration_cannot_reference_itself() {
        let env = vec![Declaration {
            ty: Expr::Const(0),
            value: None,
        }];

        assert!(!environment_well_formed(&env));
    }

    #[test]
    fn constants_compare_structurally() {
        assert_eq!(Expr::Const(2).def_eq(&Expr::Const(2)), Some(true));

        assert_eq!(Expr::Const(2).def_eq(&Expr::Const(3)), Some(false));
    }

    #[test]
    fn lookup_constant_type_returns_declared_type() {
        let env = vec![Declaration {
            ty: Expr::Sort(0),
            value: None,
        }];

        assert_eq!(lookup_constant_type(&env, 0), Some(Expr::Sort(0)));
    }

    #[test]
    fn lookup_missing_constant_fails() {
        let env = vec![Declaration {
            ty: Expr::Sort(0),
            value: None,
        }];

        assert_eq!(lookup_constant_type(&env, 1), None);
    }

    #[test]
    fn constant_type_is_inferred_from_environment() {
        let env = vec![Declaration {
            ty: Expr::Sort(0),
            value: None,
        }];

        assert_eq!(
            Expr::Const(0).infer_type_with_env(&env, &[]),
            Some(Expr::Sort(0))
        );
    }

    #[test]
    fn global_constant_survives_local_binder_depth() {
        let env = vec![
            // P : Prop
            Declaration {
                ty: Expr::Sort(0),
                value: None,
            },
        ];

        let term = Expr::Lambda(Box::new(Expr::Const(0)), Box::new(Expr::Index(0)));

        let expected = Expr::Pi(Box::new(Expr::Const(0)), Box::new(Expr::Const(0)));

        assert_eq!(term.infer_type_with_env(&env, &[]), Some(expected));
    }

    #[test]
    fn proposition_axiom_is_well_formed() {
        let env = vec![
            // P : Prop
            Declaration {
                ty: Expr::Sort(0),
                value: None,
            },
        ];

        assert!(environment_well_formed(&env));
    }

    #[test]
    fn later_declaration_can_reference_earlier_global() {
        let env = vec![
            // P : Prop
            Declaration {
                ty: Expr::Sort(0),
                value: None,
            },
            // p : P
            Declaration {
                ty: Expr::Const(0),
                value: None,
            },
        ];

        assert!(environment_well_formed(&env));
    }

    #[test]
    fn declaration_cannot_reference_future_global() {
        let env = vec![
            Declaration {
                ty: Expr::Sort(0),
                value: None,
            },
            Declaration {
                ty: Expr::Const(2),
                value: None,
            },
        ];

        assert!(!environment_well_formed(&env));
    }

    #[test]
    fn valid_definition_is_well_formed() {
        let env = vec![
            // P : Prop
            Declaration {
                ty: Expr::Sort(0),
                value: None,
            },
            // p : P
            Declaration {
                ty: Expr::Const(0),
                value: None,
            },
            // q : P := p
            Declaration {
                ty: Expr::Const(0),
                value: Some(Expr::Const(1)),
            },
        ];

        assert!(environment_well_formed(&env));
    }

    #[test]
    fn definition_body_must_match_declared_type() {
        let env = vec![
            // P : Prop
            Declaration {
                ty: Expr::Sort(0),
                value: None,
            },
            // bogus : P := Prop
            Declaration {
                ty: Expr::Const(0),
                value: Some(Expr::Sort(0)),
            },
        ];

        assert!(!environment_well_formed(&env));
    }

    #[test]
    fn check_type_with_env_rejects_malformed_environment() {
        let env = vec![Declaration {
            ty: Expr::Const(0),
            value: None,
        }];

        assert_eq!(
            Expr::Sort(0).check_type_with_env(&Expr::Sort(1), &env, &[],),
            Some(false)
        );
    }
}
