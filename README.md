# Derive enum from try into

[![](https://img.shields.io/crates/v/derive-enum-from-into)](https://crates.io/crates/derive-enum-from-into)

Implements `From` and `TryInto` for enums

```rust
use derive_enum_from_into::{EnumFrom, EnumTryInto};
use std::convert::TryInto;

#[derive(EnumFrom, EnumTryInto, PartialEq, Debug)]
enum Enum1 {
    A(i32),
    B,
}

assert_eq!(
    Enum1::from(54i32),
    Enum1::A(54i32)
);

let num: Result<i32, _> = Enum1::B.try_into();
assert!(num.is_err());
```

Ignores variants with duplicate type definitions or named fields

```compile_error
use derive_enum_from_into::{EnumFrom, EnumTryInto};
use std::convert::TryInto;

#[derive(EnumFrom, EnumTryInto, PartialEq, Debug)]
enum Enum1 {
    A(String),
    B(String),
    C { 
        something: bool 
    },
}

// Results in compile errors
let enum1: Result<String, _> = Enum1::A("Hello".to_owned()).try_into();
let enum1: Result<bool, _> = (Enum1::C { something: true }).try_into();
```

`From` can be ignored for variants with `#[from_ignore]`

`TryInto` can also be implemented for references of the enum. Specific variants can be ignored with `#[try_into_ignore]`

```rust
use derive_enum_from_into::{EnumFrom, EnumTryInto};
use std::convert::TryInto;

#[derive(EnumTryInto, PartialEq, Debug)]
#[try_into_references(&, &mut, owned)]
enum NumberOrString {
    Number(f32),
    #[try_into_ignore]
    String(String)
}

let x = NumberOrString::Number(4.);
assert_eq!(TryInto::<&f32>::try_into(&x), Ok(&4.));

// This won't compile as cannot TryInto String
// assert!(TryInto::<String>::try_into(NumberOrString::String("Hello World".to_owned())).is_err());

// `TryInto` comes in handy for `filter_map` cases
fn filter_only_f32s(iter: impl Iterator<Item=NumberOrString>) -> impl Iterator<Item=f32> {
    iter.filter_map(|item| item.try_into().ok())
}

// `TryInto` returns `self` if it does not match
assert_eq!(TryInto::<f32>::try_into(NumberOrString::String("Hello World".to_owned())), Err(NumberOrString::String("Hello World".to_owned())));
```

Note that because of the [default implementation of `TryInto`](https://doc.rust-lang.org/src/core/convert/mod.rs.html#573-582) the following will not compile. There are workarounds with wrapping X in a *newtype* pattern

```compile_error
#[derive(derive_enum_from_into::EnumTryInto)]
enum X {
    Leaf(i32),
    Nested(X)
}
```