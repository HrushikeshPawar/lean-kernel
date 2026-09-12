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

    let result = LeanExpr::BVar(0).infer_type_with_context(&[prop.clone()]);

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
