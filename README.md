# Derive enum from try into

[![](https://img.shields.io/crates/v/derive-enum-from-into)](https://crates.io/crates/derive-enum-from-into)

Implements `From` and `TryInto` for enums with single fields. Also ignores variants with duplicate type definition

```rust
use derive_enum_from_into::{EnumFrom, EnumTryInto};

#[derive(EnumFrom, EnumTryInto, PartialEq)]
enum X {
    A(i32),
    B(String),
    C(String),
    D,
    E {
        something: i32
    },
    F(Box<X>)
}

assert_eq!(
    X::from(54i32),
    X::A(54i32)
);

let nested_x: Result<Box<X>, _> = X::D.try_into();
assert_eq!(nested_x, Err(()));
let nested_x: Result<Box<X>, _> = X::F(Box::new(X::D)).try_into();
assert_eq!(nested_x, Ok(Box::new(X::D)));
```