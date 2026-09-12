use std::{io, process::ExitCode};

fn main() -> ExitCode {
    let stdin = io::stdin();
    lean_kernel::driver::run(stdin.lock())
}
