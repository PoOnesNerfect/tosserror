use std::{io, num};
use tosserror::{Error, Toss};

#[derive(Error, Toss, Debug)]
pub enum MyError {
    #[error("var1 error {val}")]
    Var1 { val: i32, source: io::Error },
    #[error("var2 error {val}")]
    Var2 {
        val: i32,
        source: num::TryFromIntError,
    },
}

#[derive(Error, Toss, Debug)]
#[prefix]
pub enum MyError2 {
    #[error("var1 error {val}")]
    Var1 { val: i32, source: io::Error },
    #[error("var2 error {val}")]
    Var2 {
        val: i32,
        source: num::TryFromIntError,
    },
}

fn io_fn() -> Result<(), io::Error> {
    Ok(())
}

fn main() -> Result<(), MyError> {
    io_fn().toss_var1(123)?;

    Ok(())
}
