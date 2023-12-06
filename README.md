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
    },
    #[error("data store disconnected with msg {msg}: {status}")]
    Disconnect{
      status: u8,
      msg: String,
      source: std::io::Error
    }
}

// uses
get_value().toss_invalid_value(123)?;

// lazily provide context
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

## How it works

`derive(Toss)` works by creating a trait for every enum variant.

For the error below:

```rust
#[derive(Error, Toss, Debug)]
pub enum DataStoreError {
    #[error("invalid value ({value}) encountered")]
    InvalidValue {
      value: i32,
      source: std::num::TryFromIntError,
    },
    #[error("data store disconnected with msg {msg}: {status}")]
    Disconnect {
      status: u8,
      msg: String,
      source: std::io::Error
    }
}
```

it will generate traits and implementations as below:

```rust
// pseudo generated code
trait TossDataStoreInvalidValue<T> {
    fn toss_invalid_value(self, value: i32) -> Result<T, MyError>;
}
impl<T> TossDataStoreInvalidValue<T> for Result<T, std::num::TryFromIntError> {
    fn toss_invalid_value(self, value: i32) -> Result<T, MyError> { ... }
}

trait TossDataStoreDisconnect<T> {
    fn toss_disconnect(self, status: u8, msg: String) -> Result<T, MyError>;
}
impl<T> TossDataStoreDisconnect<T> for Result<T, std::io::Error> {
    fn toss_disconnect(self, status: u8, msg: String) -> Result<T, MyError> { ... }
}
```

With these traits generated, you get auto-completion of `.toss_invalid_value(i32)` to any method that returns `Result<T, TryFromError>`.
Same applies to `.toss_disconnect(u8, String)` for `Result<T, io::Error>`.

If you want these traits be visible to other modules, see [#[visibility]](#visibility).

For the full generated code example, see [Generated Code from `derive(Toss)`](#generated-code-from-derivetoss).

## Why use `derive(Toss)`

#### Brevity

When I've used libraries like `thiserror`, I've noticed that handling the errors add
significant bloat to the code, and hinders readability.

You may encounter simple function calls followed by 2 to 4 lines of error handling like below:

```rust
simple_function().map_err(|e| LongError::InvalidValue {
    context1: 123u32,
    context2: "some context".to_owned(),
    source: e
})?;
```

When the error handling is longer than the procedure itself, your code's primary focus is no longer the logic; it's handling errors. 

With `derive(Toss)`, your error handling code will almost always stay in one line:

```rust
simple_function().toss_invalid_value(123u32, "some context".to_owned())?;
```

You can also pass a closure that returns the arguments to lazily evaluate context values.

```rust
simple_function().toss_invalid_value_with(|| (123u32, "some context".to_owned()))?;
```

#### Convenience with autocompletion

With `thiserror`, it may be cumbersome to type out all the characters:
1. `.map_err`
2. two `|`s
3. enum name
4. variant name
5. field names and values

I won't say these are the major hurdles of programming, but it does get pretty annoying when you have to type these over and over.

With `derive(Toss)`, you get auto-completion!

When you type `.toss_`, you will get list of suggestions that only apply to the underlying error type.

For example, if your method returns `io::Error`, the suggestion will show only the handler methods that take in `io::Error`.

## Why you may not use `derive(Toss)`

With the upsides, I admit there is a downside that may make you not want to use this library.

#### it will obscure the code

In my [reddit post](https://www.reddit.com/r/rust/comments/18a3019/thoughts_on_my_idea_for_error_handling_and/) asking for feedback on this idea,
pretty common feedback was this.

This macro creates many traits that you cannot see how it's defined or implemented. 
If you're a new contributor to some open source library, and you encountered uses of these methods here and there,
and you had no idea about `tosserror`, you could be very confused what it means, what it does and how it works.

Compared to this, `.map_err` and `.with_context` provides a clear source that you can easily see what it does and how it works.

This is a fair criticism, and, if this is a big concern, you will not want to use this library.

To many people, using `.map_err` is just not a big enough inconvenience to justify obscuring your codebase.

For those of you who still want to give it a try, here are my thoughts on this concern:
1. if you use the rust-analyzer lsp, it will pretty clearly outline the definition, arguments, and return type,
2. the library is simple enough that, newcomers to the library will pretty easily figure out what it does, or, you can just let them know that this thing exists.

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
    },
    #[error("data store disconnected with msg {msg}: {status}")]
    #[visibility(pub(crate))]
    Disconnect{
      status: u8,
      msg: String,
      source: std::io::Error
    }
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

## Credits

This library takes a lot of implementation details and code structures from [thiserror](https://crates.io/crates/thiserror),
as it is a complementary library for `thiserror` and works on the same attributes.

Special thanks to the creator and maintainers of [thiserror](https://crates.io/crates/thiserror) for creating
such a simple, clean, easy-to-read yet great library.
