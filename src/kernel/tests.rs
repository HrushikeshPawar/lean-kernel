use super::typing::imax;
use super::{LeanAxiom, LeanDeclaration, LeanDefinition, LeanExpr, Level, Name};

fn declaration(def: LeanDefinition) -> LeanDeclaration {
    LeanDeclaration::Definition(def)
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
        level_params: vec![],
        ty: LeanExpr::Sort(Level::Succ(Box::new(Level::Zero))),
        value: LeanExpr::Sort(Level::Zero),
    };

    assert_eq!(definition.check(), Ok(true));
}

#[test]
fn definition_rejects_value_with_wrong_type() {
    let definition = LeanDefinition {
        name: Name::Str(Box::new(Name::Anonymous), "bad".to_string()),
        level_params: vec![],

        // Claims: bad : Prop
        ty: LeanExpr::Sort(Level::Zero),

        // But Prop : Type, not Prop.
        value: LeanExpr::Sort(Level::Zero),
    };

    assert_eq!(definition.check(), Ok(false));
}

#[test]
fn declared_universe_parameter_is_accepted() {
    let u = Name::Str(Box::new(Name::Anonymous), "u".to_string());

    let definition = LeanDefinition {
        name: Name::Str(Box::new(Name::Anonymous), "poly".to_string()),
        level_params: vec![u.clone()],

        // poly.{u} : Sort (u + 1) := Sort u
        ty: LeanExpr::Sort(Level::Succ(Box::new(Level::Param(u.clone())))),
        value: LeanExpr::Sort(Level::Param(u)),
    };

    assert_eq!(definition.check(), Ok(true));
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
fn imax_of_zero_and_param_is_param() {
    let param = Level::Param(Name::Str(Box::new(Name::Anonymous), "u".to_string()));

    assert_eq!(imax(&Level::Zero, &param), param);
}

#[test]
fn imax_of_param_with_itself_is_param() {
    let param = Level::Param(Name::Str(Box::new(Name::Anonymous), "u".to_string()));

    assert_eq!(imax(&param, &param), param);
}

#[test]
fn imax_of_succ_param_and_param_stays_imax() {
    let u = Name::Str(Box::new(Name::Anonymous), "u".to_string());

    // 008 pressure: imax(Succ(u), u) == IMax(Succ(u), u)
    assert_eq!(
        imax(
            &Level::Succ(Box::new(Level::Param(u.clone()))),
            &Level::Param(u)
        ),
        Level::IMax(
            Box::new(Level::Succ(Box::new(Level::Param(Name::Str(
                Box::new(Name::Anonymous),
                "u".to_string()
            ))))),
            Box::new(Level::Param(Name::Str(
                Box::new(Name::Anonymous),
                "u".to_string()
            )))
        )
    );
}

#[test]
fn imax_of_succ_params_becomes_succ_max() {
    let u = Name::Str(Box::new(Name::Anonymous), "u".to_string());
    let v = Name::Str(Box::new(Name::Anonymous), "v".to_string());

    // Assuming u != v: imax(Succ(u), Succ(v)) == Succ(Max(u, v))
    assert_eq!(
        imax(
            &Level::Succ(Box::new(Level::Param(u))),
            &Level::Succ(Box::new(Level::Param(v)))
        ),
        Level::Succ(Box::new(Level::Max(
            Box::new(Level::Param(Name::Str(
                Box::new(Name::Anonymous),
                "u".to_string()
            ))),
            Box::new(Level::Param(Name::Str(
                Box::new(Name::Anonymous),
                "v".to_string()
            )))
        )))
    );
}

#[test]
fn imax_of_succ_zero_and_param_stays_imax() {
    let u = Name::Str(Box::new(Name::Anonymous), "u".to_string());

    // Guard: imax(Succ(Zero), Param(u)) must remain IMax, not become Max.
    assert_eq!(
        imax(
            &Level::Succ(Box::new(Level::Zero)),
            &Level::Param(Name::Str(Box::new(Name::Anonymous), "u".to_string()))
        ),
        Level::IMax(
            Box::new(Level::Succ(Box::new(Level::Zero))),
            Box::new(Level::Param(u))
        )
    );
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
        level_params: vec![],
        ty: LeanExpr::Sort(Level::Succ(Box::new(Level::Zero))),
        value: LeanExpr::Sort(Level::Zero),
    };

    let env = vec![declaration(foo)];

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
fn const_type_instantiates_universe_arguments() {
    let u = Name::Str(Box::new(Name::Anonymous), "u".to_string());

    // poly.{u} : Sort (u+1) := Sort u
    let poly = LeanDefinition {
        name: Name::Str(Box::new(Name::Anonymous), "poly".to_string()),
        level_params: vec![u],
        ty: LeanExpr::Sort(Level::Succ(Box::new(Level::Param(Name::Str(
            Box::new(Name::Anonymous),
            "u".to_string(),
        ))))),
        value: LeanExpr::Sort(Level::Param(Name::Str(
            Box::new(Name::Anonymous),
            "u".to_string(),
        ))),
    };

    // Const(poly, [0]) must infer Sort 1.
    let result = LeanExpr::Const(
        Name::Str(Box::new(Name::Anonymous), "poly".to_string()),
        vec![Level::Zero],
    )
    .infer_type_with_env_and_context(&[declaration(poly)], &[]);

    assert_eq!(
        result,
        Ok(LeanExpr::Sort(Level::Succ(Box::new(Level::Zero))))
    );
}

#[test]
fn const_universe_arity_mismatch_fails() {
    let u = Name::Str(Box::new(Name::Anonymous), "u".to_string());

    // poly.{u} : Sort (u+1) := Sort u
    let poly = LeanDefinition {
        name: Name::Str(Box::new(Name::Anonymous), "poly".to_string()),
        level_params: vec![u],
        ty: LeanExpr::Sort(Level::Succ(Box::new(Level::Param(Name::Str(
            Box::new(Name::Anonymous),
            "u".to_string(),
        ))))),
        value: LeanExpr::Sort(Level::Param(Name::Str(
            Box::new(Name::Anonymous),
            "u".to_string(),
        ))),
    };

    // poly.{} supplies no universe argument.
    let result = LeanExpr::Const(
        Name::Str(Box::new(Name::Anonymous), "poly".to_string()),
        vec![],
    )
    .infer_type_with_env_and_context(&[declaration(poly)], &[]);

    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Universe argument count mismatch")
    );
}

#[test]
fn whnf_instantiates_const_universe_then_delta_beta_reduces() {
    let u = Name::Str(Box::new(Name::Anonymous), "u".to_string());
    let two = Level::Succ(Box::new(Level::Succ(Box::new(Level::Zero))));

    // id.{u} := λ α : Sort u, λ a : α, a
    let id = LeanDefinition {
        name: Name::Str(Box::new(Name::Anonymous), "id".to_string()),
        level_params: vec![u],
        ty: LeanExpr::Pi(
            Box::new(LeanExpr::Sort(Level::Param(Name::Str(
                Box::new(Name::Anonymous),
                "u".to_string(),
            )))),
            Box::new(LeanExpr::Pi(
                Box::new(LeanExpr::BVar(0)),
                Box::new(LeanExpr::BVar(1)),
            )),
        ),
        value: LeanExpr::Lambda(
            Box::new(LeanExpr::Sort(Level::Param(Name::Str(
                Box::new(Name::Anonymous),
                "u".to_string(),
            )))),
            Box::new(LeanExpr::Lambda(
                Box::new(LeanExpr::BVar(0)),
                Box::new(LeanExpr::BVar(0)),
            )),
        ),
    };

    let type_ = LeanExpr::Sort(Level::Succ(Box::new(Level::Zero)));
    let prop = LeanExpr::Sort(Level::Zero);

    // id.{2} Type Prop must WHNF to Prop.
    let expr = LeanExpr::Application(
        Box::new(LeanExpr::Application(
            Box::new(LeanExpr::Const(
                Name::Str(Box::new(Name::Anonymous), "id".to_string()),
                vec![two],
            )),
            Box::new(type_),
        )),
        Box::new(prop.clone()),
    );

    assert_eq!(expr.whnf_with_env(&[declaration(id)]), Ok(prop));
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
        level_params: vec![],
        ty: type_.clone(),
        value: prop,
    };

    assert_eq!(foo.check_with_env(&[]), Ok(true));

    // bar : Type := foo
    let bar = LeanDefinition {
        name: Name::Str(Box::new(Name::Anonymous), "bar".to_string()),
        level_params: vec![],
        ty: type_,
        value: LeanExpr::Const(foo_name, vec![]),
    };

    assert_eq!(bar.check_with_env(&[declaration(foo)]), Ok(true));
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
fn pi_domain_type_acting_as_sort_via_whnf_is_accepted() {
    // id.{u} := λ α : Sort u, λ a : α, a
    let u = Name::Str(Box::new(Name::Anonymous), "u".to_string());
    let id = LeanDefinition {
        name: Name::Str(Box::new(Name::Anonymous), "id".to_string()),
        level_params: vec![u],
        ty: LeanExpr::Pi(
            Box::new(LeanExpr::Sort(Level::Param(Name::Str(
                Box::new(Name::Anonymous),
                "u".to_string(),
            )))),
            Box::new(LeanExpr::Pi(
                Box::new(LeanExpr::BVar(0)),
                Box::new(LeanExpr::BVar(1)),
            )),
        ),
        value: LeanExpr::Lambda(
            Box::new(LeanExpr::Sort(Level::Param(Name::Str(
                Box::new(Name::Anonymous),
                "u".to_string(),
            )))),
            Box::new(LeanExpr::Lambda(
                Box::new(LeanExpr::BVar(0)),
                Box::new(LeanExpr::BVar(0)),
            )),
        ),
    };

    let two = Level::Succ(Box::new(Level::Succ(Box::new(Level::Zero))));
    let type_ = LeanExpr::Sort(Level::Succ(Box::new(Level::Zero)));
    let prop = LeanExpr::Sort(Level::Zero);

    let id_type_prop = LeanExpr::Application(
        Box::new(LeanExpr::Application(
            Box::new(LeanExpr::Const(
                Name::Str(Box::new(Name::Anonymous), "id".to_string()),
                vec![two],
            )),
            Box::new(type_),
        )),
        Box::new(prop.clone()),
    );

    // forallSortWhnf : Prop := Π (p : id.{2} Type Prop), Π (a : p), p
    let definition = LeanDefinition {
        name: Name::Str(Box::new(Name::Anonymous), "forallSortWhnf".to_string()),
        level_params: vec![],
        ty: prop,
        value: LeanExpr::Pi(
            Box::new(id_type_prop),
            Box::new(LeanExpr::Pi(
                Box::new(LeanExpr::BVar(0)),
                Box::new(LeanExpr::BVar(1)),
            )),
        ),
    };

    assert_eq!(definition.check_with_env(&[declaration(id)]), Ok(true));
}

#[test]
fn whnf_delta_then_beta_reduces_const_type() {
    let type_ = LeanExpr::Sort(Level::Succ(Box::new(Level::Zero)));
    let prop = LeanExpr::Sort(Level::Zero);

    let name = Name::Str(Box::new(Name::Anonymous), "constType".to_string());

    let definition = LeanDefinition {
        name: name.clone(),
        level_params: vec![],
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

    assert_eq!(expr.whnf_with_env(&[declaration(definition)]), Ok(prop));
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
        level_params: vec![],
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

    assert_eq!(
        const_type_application(&prop)
            .def_eq_with_env(&prop, &[declaration(const_type_definition())]),
        Ok(true)
    );
}

#[test]
fn def_eq_recurses_under_pi() {
    let prop = LeanExpr::Sort(Level::Zero);

    let lhs = LeanExpr::Pi(
        Box::new(prop.clone()),
        Box::new(const_type_application(&prop)),
    );
    let rhs = LeanExpr::Pi(Box::new(prop.clone()), Box::new(prop.clone()));

    assert_eq!(
        lhs.def_eq_with_env(&rhs, &[declaration(const_type_definition())]),
        Ok(true)
    );
}

#[test]
fn def_eq_distinguishes_pi_codomain_levels() {
    let prop = LeanExpr::Sort(Level::Zero);
    let type_ = LeanExpr::Sort(Level::Succ(Box::new(Level::Zero)));

    let lhs = LeanExpr::Pi(Box::new(prop.clone()), Box::new(prop.clone()));
    let rhs = LeanExpr::Pi(Box::new(prop), Box::new(type_));

    assert_eq!(lhs.def_eq_with_env(&rhs, &[]), Ok(false));
}

#[test]
fn definition_rejects_pi_domain_that_is_not_a_type() {
    let prop = LeanExpr::Sort(Level::Zero);
    let x = Name::Str(Box::new(Name::Anonymous), "x".to_string());
    let h = Name::Str(Box::new(Name::Anonymous), "h".to_string());

    // outer x : Prop := Prop -> Prop (a genuine proposition)
    let outer = LeanDefinition {
        name: x.clone(),
        level_params: vec![],
        ty: prop.clone(),
        value: LeanExpr::Pi(Box::new(prop.clone()), Box::new(LeanExpr::BVar(0))),
    };

    // inner proof h : x (axiom-style entry: no Lean-unfoldable value)
    let proof = LeanDefinition {
        name: h.clone(),
        level_params: vec![],
        ty: LeanExpr::Const(x.clone(), vec![]),
        value: LeanExpr::Const(x.clone(), vec![]),
    };

    // forallBad : Prop := Π (_ : h), _ — the domain `h` is a proof term,
    // whose type is `x`, not a Sort; the binder must be rejected.
    let definition = LeanDefinition {
        name: Name::Str(Box::new(Name::Anonymous), "forallBad".to_string()),
        level_params: vec![],
        ty: prop,
        value: LeanExpr::Pi(
            Box::new(LeanExpr::Const(h, vec![])),
            Box::new(LeanExpr::BVar(0)),
        ),
    };

    assert_eq!(outer.check_with_env(&[]), Ok(true));
    assert_eq!(
        definition.check_with_env(&[declaration(outer), declaration(proof)]),
        Ok(false)
    );
}

#[test]
fn axiom_constant_infers_declared_type() {
    // axiom A : Sort 0
    let axiom = LeanAxiom {
        name: Name::Str(Box::new(Name::Anonymous), "A".to_string()),
        level_params: vec![],
        ty: LeanExpr::Sort(Level::Zero),
    };

    let env = vec![LeanDeclaration::Axiom(axiom)];

    let result = LeanExpr::Const(
        Name::Str(Box::new(Name::Anonymous), "A".to_string()),
        vec![],
    )
    .infer_type_with_env_and_context(&env, &[]);

    assert_eq!(result, Ok(LeanExpr::Sort(Level::Zero)));
}

#[test]
fn axiom_is_not_reducible() {
    // axiom A : Sort 0
    let axiom = LeanAxiom {
        name: Name::Str(Box::new(Name::Anonymous), "A".to_string()),
        level_params: vec![],
        ty: LeanExpr::Sort(Level::Zero),
    };

    let env = vec![LeanDeclaration::Axiom(axiom)];

    let const_a = LeanExpr::Const(
        Name::Str(Box::new(Name::Anonymous), "A".to_string()),
        vec![],
    );

    assert_eq!(const_a.whnf_with_env(&env), Ok(const_a));
}

#[test]
fn axiom_with_sort_type_checks() {
    // axiom A : Sort 1  (a non-Prop axiom, e.g. nonTypeAxiom : Type)
    let axiom = LeanAxiom {
        name: Name::Str(Box::new(Name::Anonymous), "A".to_string()),
        level_params: vec![],
        ty: LeanExpr::Sort(Level::Succ(Box::new(Level::Zero))),
    };

    assert_eq!(axiom.check_with_env(&[]), Ok(true));
}

#[test]
fn axiom_whose_type_is_a_function_term_is_rejected() {
    // constType : Sort 1 -> Sort 1 -> Sort 1   (a defined function-shaped term)
    // axiom Bad : constType
    let const_a = LeanExpr::Const(
        Name::Str(Box::new(Name::Anonymous), "constType".to_string()),
        vec![],
    );

    let axiom = LeanAxiom {
        name: Name::Str(Box::new(Name::Anonymous), "Bad".to_string()),
        level_params: vec![],
        ty: const_a,
    };

    // infer_type(Const constType) = Pi(...), which is not a Sort.
    let inferred = LeanExpr::Const(
        Name::Str(Box::new(Name::Anonymous), "constType".to_string()),
        vec![],
    )
    .infer_type_with_env_and_context(&[declaration(const_type_definition())], &[])
    .unwrap();
    assert!(matches!(inferred, LeanExpr::Pi(_, _)));

    assert_eq!(
        axiom.check_with_env(&[declaration(const_type_definition())]),
        Ok(false)
    );
}

#[test]
fn level_equiv_imax_zero() {
    let one = Level::Succ(Box::new(Level::Zero));
    let imax_one_zero = Level::IMax(Box::new(one), Box::new(Level::Zero));
    assert!(imax_one_zero.is_equiv(&Level::Zero));
}

#[test]
fn level_equiv_imax_succ() {
    let zero = Level::Zero;
    let one = Level::Succ(Box::new(Level::Zero));
    let two = Level::Succ(Box::new(one.clone()));

    // imax(0, 1) == 1
    let imax_zero_one = Level::IMax(Box::new(zero), Box::new(one.clone()));
    assert!(imax_zero_one.is_equiv(&one));

    // imax(2, 1) == 2
    let imax_two_one = Level::IMax(Box::new(two.clone()), Box::new(one));
    assert!(imax_two_one.is_equiv(&two));
}

#[test]
fn level_equiv_max_comm() {
    let u = Level::Param(Name::Str(Box::new(Name::Anonymous), "u".to_string()));
    let v = Level::Param(Name::Str(Box::new(Name::Anonymous), "v".to_string()));

    let uv = Level::Max(Box::new(u.clone()), Box::new(v.clone()));
    let vu = Level::Max(Box::new(v), Box::new(u));

    assert!(uv.is_equiv(&vu));
}

#[test]
fn level_equiv_max_assoc() {
    let u = Level::Param(Name::Str(Box::new(Name::Anonymous), "u".to_string()));
    let v = Level::Param(Name::Str(Box::new(Name::Anonymous), "v".to_string()));
    let w = Level::Param(Name::Str(Box::new(Name::Anonymous), "w".to_string()));

    let u_vw = Level::Max(
        Box::new(u.clone()),
        Box::new(Level::Max(Box::new(v.clone()), Box::new(w.clone()))),
    );
    let uv_w = Level::Max(
        Box::new(Level::Max(Box::new(u), Box::new(v))),
        Box::new(w),
    );

    assert!(u_vw.is_equiv(&uv_w));
}

#[test]
fn level_equiv_max_idem() {
    let u = Level::Param(Name::Str(Box::new(Name::Anonymous), "u".to_string()));
    let uu = Level::Max(Box::new(u.clone()), Box::new(u.clone()));

    assert!(uu.is_equiv(&u));
}

#[test]
fn level_equiv_max_absorb() {
    let u = Level::Param(Name::Str(Box::new(Name::Anonymous), "u".to_string()));
    let v = Level::Param(Name::Str(Box::new(Name::Anonymous), "v".to_string()));

    let uv = Level::Max(Box::new(u.clone()), Box::new(v.clone()));
    let u_uv = Level::Max(Box::new(u), Box::new(uv.clone()));

    assert!(u_uv.is_equiv(&uv));
}

#[test]
fn level_equiv_succ_max_distrib() {
    let u = Level::Param(Name::Str(Box::new(Name::Anonymous), "u".to_string()));
    let v = Level::Param(Name::Str(Box::new(Name::Anonymous), "v".to_string()));

    let succ_max = Level::Succ(Box::new(Level::Max(Box::new(u.clone()), Box::new(v.clone()))));
    let max_succ = Level::Max(
        Box::new(Level::Succ(Box::new(u))),
        Box::new(Level::Succ(Box::new(v))),
    );

    assert!(succ_max.is_equiv(&max_succ));
}

#[test]
fn level_equiv_max_offset_absorb() {
    let u = Level::Param(Name::Str(Box::new(Name::Anonymous), "u".to_string()));
    let succ_u = Level::Succ(Box::new(u));
    let one = Level::Succ(Box::new(Level::Zero));

    // max(1, succ(u)) == succ(u) since u >= 0 implies succ(u) >= 1
    let max_one_succ_u = Level::Max(Box::new(one), Box::new(succ_u.clone()));
    assert!(max_one_succ_u.is_equiv(&succ_u));
}

#[test]
fn def_eq_sort_with_equivalent_levels() {
    let u = Level::Param(Name::Str(Box::new(Name::Anonymous), "u".to_string()));
    let v = Level::Param(Name::Str(Box::new(Name::Anonymous), "v".to_string()));

    let sort1 = LeanExpr::Sort(Level::Max(Box::new(u.clone()), Box::new(v.clone())));
    let sort2 = LeanExpr::Sort(Level::Max(Box::new(v), Box::new(u)));

    assert_eq!(sort1.def_eq_with_env(&sort2, &[]), Ok(true));
}

#[test]
fn def_eq_const_with_equivalent_levels() {
    let u = Level::Param(Name::Str(Box::new(Name::Anonymous), "u".to_string()));
    let v = Level::Param(Name::Str(Box::new(Name::Anonymous), "v".to_string()));

    let c1 = LeanExpr::Const(
        Name::Str(Box::new(Name::Anonymous), "my_const".to_string()),
        vec![Level::Max(Box::new(u.clone()), Box::new(v.clone()))],
    );
    let c2 = LeanExpr::Const(
        Name::Str(Box::new(Name::Anonymous), "my_const".to_string()),
        vec![Level::Max(Box::new(v), Box::new(u))],
    );

    assert_eq!(c1.def_eq_with_env(&c2, &[]), Ok(true));
}
