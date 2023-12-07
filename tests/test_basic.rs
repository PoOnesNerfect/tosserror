use std::{io, num::TryFromIntError};
use thiserror::Error;
use tosserror::Toss;

#[derive(Debug, Error, Toss)]
#[error("struct error")]
struct StructError {
    msg: String,
    source: io::Error,
}

#[derive(Debug, Error, Toss)]
#[error("struct error")]
struct TupleError(String, #[source] io::Error, i32);

#[derive(Debug, Error, Toss)]
enum EnumError {
    #[error("io error")]
    #[prefix(connect)]
    IoError { msg: String, source: io::Error },
    #[error("invalid value: {value}")]
    #[visibility(pub(crate))]
    InvalidValue { value: i32, source: TryFromIntError },
    #[error("tuple variant")]
    TupleVariant(i32, #[source] TupleError, String),
}

fn io_fn() -> Result<(), io::Error> {
    Ok(())
}

fn convert_fn() -> Result<(), TryFromIntError> {
    Ok(())
}

fn tuple_fn() -> Result<(), TupleError> {
    Ok(())
}

#[test]
fn test_struct() -> Result<(), StructError> {
    // handling with map_err
    io_fn().map_err(|e| StructError {
        msg: "msg".to_owned(),
        source: e,
    })?;

    // handling with maperror
    io_fn().toss_struct_with(|| "msg".to_owned())?;

    Ok(())
}

#[test]
fn test_tuple() -> Result<(), TupleError> {
    // handling with map_err
    io_fn().map_err(|e| TupleError("msg".to_owned(), e, 123))?;

    // handling with maperror
    io_fn().toss_tuple_with(|| ("msg".to_owned(), 123))?;

    Ok(())
}

#[test]
fn test_enum() -> Result<(), EnumError> {
    // 1.
    // handling with map_err
    io_fn().map_err(|e| EnumError::IoError {
        msg: "msg".to_owned(),
        source: e,
    })?;

    // handling with maperror
    io_fn().toss_connect_io_with(|| "msg".to_owned())?;

    // 2.
    // handling with map_err
    convert_fn().map_err(|e| EnumError::InvalidValue {
        value: 123,
        source: e,
    })?;

    // handling with maperror
    convert_fn().toss_invalid_value(123)?;

    // 3.
    // handling with map_err
    tuple_fn().map_err(|e| EnumError::TupleVariant(123, e, "some msg".to_owned()))?;

    // handling with maperror
    tuple_fn().toss_tuple_variant(123, "some msg".to_owned())?;

    Ok(())
}
