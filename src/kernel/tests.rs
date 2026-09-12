use super::typing::imax;
use super::{LeanDefinition, LeanExpr, Level, Name};

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

#[test]
fn bvar_zero_gets_nearest_binder_type() {
    let prop = LeanExpr::Sort(Level::Zero);

    let result = LeanExpr::BVar(0).infer_type_with_context(std::slice::from_ref(&prop));

    assert_eq!(result, Ok(prop));
}

#[test]
fn bvar_out_of_scope_is_rejected() {
    let result = LeanExpr::BVar(0).infer_type_with_context(&[]);

    assert!(result.is_err());
}

#[test]
fn dependent_prop_pi_has_type_prop() {
    let prop = LeanExpr::Sort(Level::Zero);

    let pi = LeanExpr::Pi(Box::new(prop), Box::new(LeanExpr::BVar(0)));

    assert_eq!(pi.infer_type(), Ok(LeanExpr::Sort(Level::Zero)));
}

#[test]
fn arrow_prop_pi_still_has_type_type() {
    let prop = LeanExpr::Sort(Level::Zero);

    let pi = LeanExpr::Pi(Box::new(prop.clone()), Box::new(prop));

    assert_eq!(
        pi.infer_type(),
        Ok(LeanExpr::Sort(Level::Succ(Box::new(Level::Zero))))
    );
}

#[test]
fn bvar_lookup_lifts_stored_dependent_type() {
    let context = vec![LeanExpr::BVar(0), LeanExpr::Sort(Level::Zero)];

    let result = LeanExpr::BVar(0).infer_type_with_context(&context);

    assert_eq!(result, Ok(LeanExpr::BVar(1)));
}

#[test]
fn lift_bvar_at_cutoff() {
    assert_eq!(LeanExpr::BVar(0).lift(1, 0).unwrap(), LeanExpr::BVar(1));
}

#[test]
fn lift_bvar_below_cutoff_is_unchanged() {
    assert_eq!(LeanExpr::BVar(0).lift(1, 1).unwrap(), LeanExpr::BVar(0));
}

#[test]
fn lift_pi_increments_cutoff_in_body() {
    let expr = LeanExpr::Pi(Box::new(LeanExpr::BVar(0)), Box::new(LeanExpr::BVar(0)));

    assert_eq!(
        expr.lift(1, 0).unwrap(),
        LeanExpr::Pi(Box::new(LeanExpr::BVar(1)), Box::new(LeanExpr::BVar(0)),)
    );
}

#[test]
fn lift_lambda_increments_cutoff_in_body() {
    let expr = LeanExpr::Lambda(Box::new(LeanExpr::BVar(0)), Box::new(LeanExpr::BVar(0)));

    assert_eq!(
        expr.lift(1, 0).unwrap(),
        LeanExpr::Lambda(Box::new(LeanExpr::BVar(1)), Box::new(LeanExpr::BVar(0)),)
    );
}

#[test]
fn lambda_identity_infers_pi_type() {
    let prop = LeanExpr::Sort(Level::Zero);

    let lambda = LeanExpr::Lambda(Box::new(prop.clone()), Box::new(LeanExpr::BVar(0)));

    assert_eq!(
        lambda.infer_type(),
        Ok(LeanExpr::Pi(Box::new(prop.clone()), Box::new(prop),))
    );
}

#[test]
fn nested_lambda_can_reference_outer_binder() {
    let type_ = LeanExpr::Sort(Level::Succ(Box::new(Level::Zero)));

    let lambda = LeanExpr::Lambda(
        Box::new(type_.clone()),
        Box::new(LeanExpr::Lambda(
            Box::new(type_.clone()),
            Box::new(LeanExpr::BVar(1)),
        )),
    );

    assert_eq!(
        lambda.infer_type(),
        Ok(LeanExpr::Pi(
            Box::new(type_.clone()),
            Box::new(LeanExpr::Pi(Box::new(type_.clone()), Box::new(type_),)),
        ))
    );
}

#[test]
fn lambda_rejects_parameter_annotation_that_is_not_a_type() {
    let prop = LeanExpr::Sort(Level::Zero);

    // This is a term:
    // λ (p : Prop), p
    //
    // Its inferred type is Pi(Prop, Prop), not Sort(_),
    // so it cannot itself be used as a binder type annotation.
    let term_as_annotation = LeanExpr::Lambda(Box::new(prop), Box::new(LeanExpr::BVar(0)));

    let lambda = LeanExpr::Lambda(Box::new(term_as_annotation), Box::new(LeanExpr::BVar(0)));

    assert!(lambda.infer_type().is_err());
}

#[test]
fn lift_application_recurses_without_changing_cutoff() {
    let expr = LeanExpr::Application(Box::new(LeanExpr::BVar(0)), Box::new(LeanExpr::BVar(1)));

    assert_eq!(
        expr.lift(1, 1).unwrap(),
        LeanExpr::Application(Box::new(LeanExpr::BVar(0)), Box::new(LeanExpr::BVar(2)),)
    );
}

#[test]
fn dependent_application_succeeds() {
    // (λ A : Type, λ x : A, x) Prop
    let type_ = LeanExpr::Sort(Level::Succ(Box::new(Level::Zero)));
    let prop = LeanExpr::Sort(Level::Zero);

    let lambda = LeanExpr::Lambda(
        Box::new(type_),
        Box::new(LeanExpr::Lambda(
            Box::new(LeanExpr::BVar(0)),
            Box::new(LeanExpr::BVar(0)),
        )),
    );

    // Prop's type is Type, which matches the domain.
    let applied = LeanExpr::Application(Box::new(lambda), Box::new(prop.clone()));

    assert_eq!(
        applied.infer_type(),
        Ok(LeanExpr::Pi(Box::new(prop.clone()), Box::new(prop),))
    );
}

#[test]
fn non_function_application_fails() {
    // Prop Prop — Prop : Type, not a Pi.
    let prop = LeanExpr::Sort(Level::Zero);

    let applied = LeanExpr::Application(Box::new(prop.clone()), Box::new(prop));

    assert!(applied.infer_type().is_err());
}

#[test]
fn constant_infers_type_from_environment() {
    // foo : Type := Prop
    let foo = LeanDefinition {
        name: Name::Str(Box::new(Name::Anonymous), "foo".to_string()),
        ty: LeanExpr::Sort(Level::Succ(Box::new(Level::Zero))),
        value: LeanExpr::Sort(Level::Zero),
    };

    let env = vec![foo];

    let result = LeanExpr::Const(
        Name::Str(Box::new(Name::Anonymous), "foo".to_string()),
        vec![],
    )
    .infer_type_with_env_and_context(&env, &[]);

    assert_eq!(
        result,
        Ok(LeanExpr::Sort(Level::Succ(Box::new(Level::Zero))))
    );
}

#[test]
fn unknown_constant_fails() {
    let result = LeanExpr::Const(
        Name::Str(Box::new(Name::Anonymous), "foo".to_string()),
        vec![],
    )
    .infer_type_with_env_and_context(&[], &[]);

    assert!(result.is_err());
}

#[test]
fn const_with_universe_levels_fails() {
    let result = LeanExpr::Const(
        Name::Str(Box::new(Name::Anonymous), "foo".to_string()),
        vec![Level::Succ(Box::new(Level::Zero))],
    )
    .infer_type_with_env_and_context(&[], &[]);

    assert!(result.is_err());
}

#[test]
fn instantiate_bvar0_replaces_target() {
    let replacement = LeanExpr::Sort(Level::Zero);

    assert_eq!(
        LeanExpr::BVar(0).instantiate_bvar0(&replacement).unwrap(),
        replacement
    );
}

#[test]
fn instantiate_bvar0_decrements_outer_reference() {
    let replacement = LeanExpr::Sort(Level::Zero);

    assert_eq!(
        LeanExpr::BVar(1).instantiate_bvar0(&replacement).unwrap(),
        LeanExpr::BVar(0)
    );
}

#[test]
fn instantiate_bvar0_avoids_capture_under_lambda() {
    let prop = LeanExpr::Sort(Level::Zero);

    let body = LeanExpr::Lambda(Box::new(prop.clone()), Box::new(LeanExpr::BVar(1)));

    let replacement = LeanExpr::BVar(0);

    assert_eq!(
        body.instantiate_bvar0(&replacement).unwrap(),
        LeanExpr::Lambda(Box::new(prop), Box::new(LeanExpr::BVar(1)),)
    );
}

#[test]
fn later_definition_can_reference_prior_definition() {
    let type_ = LeanExpr::Sort(Level::Succ(Box::new(Level::Zero)));
    let prop = LeanExpr::Sort(Level::Zero);

    let foo_name = Name::Str(Box::new(Name::Anonymous), "foo".to_string());

    let foo = LeanDefinition {
        name: foo_name.clone(),
        ty: type_.clone(),
        value: prop,
    };

    assert_eq!(foo.check_with_env(&[]), Ok(true));

    // bar : Type := foo
    let bar = LeanDefinition {
        name: Name::Str(Box::new(Name::Anonymous), "bar".to_string()),
        ty: type_,
        value: LeanExpr::Const(foo_name, vec![]),
    };

    assert_eq!(bar.check_with_env(&[foo]), Ok(true));
}

#[test]
fn whnf_beta_reduces_head_application() {
    let prop = LeanExpr::Sort(Level::Zero);

    let expr = LeanExpr::Application(
        Box::new(LeanExpr::Lambda(
            Box::new(prop.clone()),
            Box::new(LeanExpr::BVar(0)),
        )),
        Box::new(prop.clone()),
    );

    assert_eq!(expr.whnf_with_env(&[]), Ok(prop));
}

#[test]
fn whnf_delta_then_beta_reduces_const_type() {
    let type_ = LeanExpr::Sort(Level::Succ(Box::new(Level::Zero)));
    let prop = LeanExpr::Sort(Level::Zero);

    let name = Name::Str(Box::new(Name::Anonymous), "constType".to_string());

    let definition = LeanDefinition {
        name: name.clone(),
        ty: LeanExpr::Pi(
            Box::new(type_.clone()),
            Box::new(LeanExpr::Pi(Box::new(type_.clone()), Box::new(type_))),
        ),
        value: LeanExpr::Lambda(
            Box::new(LeanExpr::Sort(Level::Succ(Box::new(Level::Zero)))),
            Box::new(LeanExpr::Lambda(
                Box::new(LeanExpr::Sort(Level::Succ(Box::new(Level::Zero)))),
                Box::new(LeanExpr::BVar(1)),
            )),
        ),
    };

    let prop_to_prop = LeanExpr::Pi(Box::new(prop.clone()), Box::new(prop.clone()));

    let expr = LeanExpr::Application(
        Box::new(LeanExpr::Application(
            Box::new(LeanExpr::Const(name, vec![])),
            Box::new(prop.clone()),
        )),
        Box::new(prop_to_prop),
    );

    assert_eq!(expr.whnf_with_env(&[definition]), Ok(prop));
}

#[test]
fn whnf_does_not_reduce_under_pi() {
    let prop = LeanExpr::Sort(Level::Zero);

    // (λ x : Prop, x) Prop in the codomain
    let redex = LeanExpr::Application(
        Box::new(LeanExpr::Lambda(
            Box::new(prop.clone()),
            Box::new(LeanExpr::BVar(0)),
        )),
        Box::new(prop.clone()),
    );

    let pi = LeanExpr::Pi(Box::new(prop), Box::new(redex));

    assert_eq!(pi.whnf_with_env(&[]), Ok(pi));
}

fn const_type_definition() -> LeanDefinition {
    let type_ = LeanExpr::Sort(Level::Succ(Box::new(Level::Zero)));

    LeanDefinition {
        name: Name::Str(Box::new(Name::Anonymous), "constType".to_string()),
        ty: LeanExpr::Pi(
            Box::new(type_.clone()),
            Box::new(LeanExpr::Pi(Box::new(type_.clone()), Box::new(type_))),
        ),
        value: LeanExpr::Lambda(
            Box::new(LeanExpr::Sort(Level::Succ(Box::new(Level::Zero)))),
            Box::new(LeanExpr::Lambda(
                Box::new(LeanExpr::Sort(Level::Succ(Box::new(Level::Zero)))),
                Box::new(LeanExpr::BVar(1)),
            )),
        ),
    }
}

fn const_type_application(prop: &LeanExpr) -> LeanExpr {
    LeanExpr::Application(
        Box::new(LeanExpr::Application(
            Box::new(LeanExpr::Const(
                Name::Str(Box::new(Name::Anonymous), "constType".to_string()),
                vec![],
            )),
            Box::new(prop.clone()),
        )),
        Box::new(LeanExpr::Pi(Box::new(prop.clone()), Box::new(prop.clone()))),
    )
}

#[test]
fn def_eq_reduces_const_application_to_prop() {
    let prop = LeanExpr::Sort(Level::Zero);
    let env = vec![const_type_definition()];

    assert_eq!(
        const_type_application(&prop).def_eq_with_env(&prop, &env),
        Ok(true)
    );
}

#[test]
fn def_eq_recurses_under_pi() {
    let prop = LeanExpr::Sort(Level::Zero);
    let env = vec![const_type_definition()];

    let lhs = LeanExpr::Pi(
        Box::new(prop.clone()),
        Box::new(const_type_application(&prop)),
    );
    let rhs = LeanExpr::Pi(Box::new(prop.clone()), Box::new(prop.clone()));

    assert_eq!(lhs.def_eq_with_env(&rhs, &env), Ok(true));
}

#[test]
fn def_eq_distinguishes_pi_codomain_levels() {
    let prop = LeanExpr::Sort(Level::Zero);
    let type_ = LeanExpr::Sort(Level::Succ(Box::new(Level::Zero)));

    let lhs = LeanExpr::Pi(Box::new(prop.clone()), Box::new(prop.clone()));
    let rhs = LeanExpr::Pi(Box::new(prop), Box::new(type_));

    assert_eq!(lhs.def_eq_with_env(&rhs, &[]), Ok(false));
}
