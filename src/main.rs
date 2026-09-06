use lean_kernel::Term::*;
use lean_kernel::Type::*;

fn main() {
    // println!("λ.0 => {:?}", Lambda(Box::new(Index(0))));
    // println!(
    //     "λ.λ.1 => {:?}",
    //     Lambda(Box::new(Lambda(Box::new(Index(1)))))
    // );
    // println!(
    //     "λ.λ.(1 0) => {:?}",
    //     Lambda(Box::new(Lambda(Box::new(Application(
    //         Box::new(Index(1)),
    //         Box::new(Index(0))
    //     )))))
    // );
    // println!(
    //     "(λ.0)(λ.0) => {:?}",
    //     Application(
    //         Box::new(Lambda(Box::new(Index(0)))),
    //         Box::new(Lambda(Box::new(Index(0))))
    //     )
    // );

    // let id_function = Lambda(Box::new(Index(0)));
    // println!("Nodes in ID function: {}", id_function.count_nodes()); // Outputs: 2

    // let sample_function = Lambda(Box::new(Lambda(Box::new(Application(
    //     Box::new(Index(0)),
    //     Box::new(Index(1)),
    // )))));
    // println!(
    //     "Nodes in sample function [λ.λ.(0 1)]: {}",
    //     sample_function.count_nodes()
    // );

    // Shift
    let sample_term_1 = Lambda(
        Base,
        Box::new(Application(Box::new(Index(1)), Box::new(Index(0)))),
    );
    println!("Shift Term `λ.(1 0)` by amount=1 and cutoff=0 -> {:?}", {
        sample_term_1.shift(1, 0)
    });

    let sample_term_2 = Lambda(
        Base,
        Box::new(Lambda(
            Base,
            Box::new(Application(
                Box::new(Application(Box::new(Index(2)), Box::new(Index(1)))),
                Box::new(Index(0)),
            )),
        )),
    );
    println!(
        "Shift Term `λ.λ.((2 1) 0)` by amount=1 and cutoff=0 -> {:?}",
        { sample_term_2.shift(1, 0) }
    );

    let sample_term_3 = Application(Box::new(Index(1)), Box::new(Index(0)));
    println!("Shift Term `(1 0)` by amount=2 and cutoff=0 -> {:?}", {
        sample_term_3.shift(2, 0)
    });

    // Type
    let identity = Lambda(Base, Box::new(Index(0)));
    println!("{identity:?}");
}
