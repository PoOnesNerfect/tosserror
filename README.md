derive(Toss)
=============

Tiny helper library for [thiserror](https://crates.io/crates/thiserror) that generates traits to conveniently handle errors.

```toml
[dependencies]
tosserror = "0.1"
```

*Compiler support: requires rustc 1.56+*

<br>

## Example Usage

```rust
use thiserror::Error;
use tosserror::Toss;

#[derive(Error, Toss, Debug)]
pub enum DataStoreError {
    #[error("invalid value ({value}) encountered")]
    InvalidValue {
      value: i32,
      source: std::num::TryFromIntError,
    }
    #[error("data store disconnected with msg {msg}: {status}")]
    Disconnect{
      status: u8,
      msg: String,
      source: std::io::Error
    },
}

// uses
get_value().toss_invalid_value(123)?;

data_store_fn().toss_disconnect_with(|| (123, "some msg".to_owned()))?;
```

#### Comparison with conventional `map_err`

```rust
data_store_fn().map_err(|e| DataStoreError::Disconnect {
    status: 123,
    msg: "some msg".to_owned(),
    source: e
})?;

get_value().map_err(|e| DataStoreError::InvalidValue {
  value: 123,
  source: e
})?;
```

<br>

## Why use `derive(Toss)`

### Who shouldn't use `derive(Toss)`

#### You don't like magic

## Attributes

### thiserror's attributes: `#[source]`, `#[from]`, `#[backtrace]`

The library uses the same rules as thiserror to determine the source and backtrace fields.

If you declare a field as source or backtrace, either by field name or attribute, it will be excluded from the generated method's arguments.

```rust
#[derive(Error, Toss, Debug)]
#[error("my error with value {value}")]
pub struct MyError {
  value: i32,
  source: io::Error,
  backtrace: std::backtrace::Backtrace
}

// pseudo generated code
trait TossMyError<T> {
    fn toss_invalid_value(self, value: i32) -> Result<T, MyError>;
}
impl<T> TossMyError<T> for io::Error {
    fn toss_invalid_value(self, value: i32) -> Result<T, MyError> {
      ...
    }
}
```

### `#[visibility]`

By default, generated traits are private, only visible to the module it's created in.

With `#[visibility]`, you can expose the generated traits to other modules or to public.

You can either place the attribute above the enum to apply to all the traits generated for the error,
or place it above specific variants to apply it to specific variants' generated traits.

#### Examples

```rust
#[derive(Error, Toss, Debug)]
#[error("...")]
#[visibility(pub)]
pub enum Error1 {
  Var1 { ... }, // generates trait `pub trait TossError1Var1`
  Var2 { ... }  // generates trait `pub trait TossError1Var2`
}

#[derive(Error, Toss, Debug)]
#[error("...")]
#[visibility(pub(super))]
pub enum Error2 {
  Var1 { ... }, // generates trait `pub(super) trait TossError2Var1`
  #[visibility(pub(crate))]
  Var2 { ... }  // generates trait `pub(crate) trait TossError2Var2`
}
```

## Comparison with `thiserror` and `snafu`

## Generated Code from `derive(Toss)`

#### Example error

```rust
use thiserror::Error;
use tosserror::Toss;

#[derive(Error, Toss, Debug)]
pub enum DataStoreError {
    #[error("invalid value ({value}) encountered")]
    InvalidValue {
      value: i32,
      source: std::num::TryFromIntError,
    }
    #[error("data store disconnected with msg {msg}: {status}")]
    #[visibility(pub(crate))]
    Disconnect{
      status: u8,
      msg: String,
      source: std::io::Error
    },
}
```

#### Generated code

```rust
trait TossDataStoreErrorInvalidValue<__RETURN> {
    fn toss_invalid_value(self, value: i32) -> Result<__RETURN, DataStoreError>;
    fn toss_invalid_value_with<F: FnOnce() -> (i32)>(
        self,
        f: F,
    ) -> Result<__RETURN, DataStoreError>;
}
impl<__RETURN> TossDataStoreErrorInvalidValue<__RETURN>
for Result<__RETURN, std::num::TryFromIntError> {
    fn toss_invalid_value(self, value: i32) -> Result<__RETURN, DataStoreError> {
        self.map_err(|e| {
            DataStoreError::InvalidValue {
                source: e,
                value,
            }
        })
    }
    fn toss_invalid_value_with<F: FnOnce() -> (i32)>(
        self,
        f: F,
    ) -> Result<__RETURN, DataStoreError> {
        self.map_err(|e| {
            let (value) = f();
            DataStoreError::InvalidValue {
                source: e,
                value,
            }
        })
    }
}

pub(crate) trait TossDataStoreErrorDisconnect<__RETURN> {
    fn toss_disconnect(
        self,
        status: u8,
        msg: String,
    ) -> Result<__RETURN, DataStoreError>;
    fn toss_disconnect_with<F: FnOnce() -> (u8, String)>(
        self,
        f: F,
    ) -> Result<__RETURN, DataStoreError>;
}
impl<__RETURN> TossDataStoreErrorDisconnect<__RETURN>
for Result<__RETURN, std::io::Error> {
    fn toss_disconnect(
        self,
        status: u8,
        msg: String,
    ) -> Result<__RETURN, DataStoreError> {
        self.map_err(|e| {
            DataStoreError::Disconnect {
                source: e,
                status,
                msg,
            }
        })
    }
    fn toss_disconnect_with<F: FnOnce() -> (u8, String)>(
        self,
        f: F,
    ) -> Result<__RETURN, DataStoreError> {
        self.map_err(|e| {
            let (status, msg) = f();
            DataStoreError::Disconnect {
                source: e,
                status,
                msg,
            }
        })
    }
}
```
