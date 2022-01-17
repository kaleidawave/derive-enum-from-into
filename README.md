# Derive enum from try into

[![](https://img.shields.io/crates/v/derive-enum-from-into)](https://crates.io/crates/derive-enum-from-into)

Implements `From` and `TryInto` for enums with single fields. Also ignores variants with duplicate type definitions

```rust
use derive_enum_from_into::{EnumFrom, EnumTryInto};

#[derive(EnumFrom, EnumTryInto, PartialEq)]
enum Enum1 {
    A(i32),
    B(String),
    C(String),
    D,
    E {
        something: i32
    },
    F(Box<Enum1>)
}

assert_eq!(
    Enum1::from(54i32),
    Enum1::A(54i32)
);

let nested_x: Result<Box<Enum1>, _> = Enum1::D.try_into();
assert!(nested_x.is_err());
let nested_x: Result<Box<Enum1>, _> = Enum1::F(Box::new(Enum1::D)).try_into();
assert_eq!(nested_x, Ok(Box::new(Enum1::D)));
```

`From` can be ignored for variants with `#[from_ignore]`

`TryInto` can also be implemented for references of the enum. Specific variants can be ignored with `#[try_into_ignore]`

```rust
#[derive(EnumTryInto, PartialEq)]
#[try_into_references(&, &mut, owned)]
enum NumberOrString {
    Number(i32),
    #[try_into_ignore]
    String(String)
}

let x = NumberOrString::Number(4);
assert_eq!(TryInto::<&i32>::try_into(&x), Ok(&4));
assert!(TryInto::<String>::try_into(NumberOrString::String("Hello World".to_owned())).is_err());
```

`TryInto` comes in handy for `filter_map` cases

```rust
fn filter_only_f32s(iter: impl Iterator<Item=NumberOrString>) -> impl Iterator<Item=f32> {
    iter.filter_map(|item| item.try_into().ok())
}
```

`TryInto` returns `self` if it does not match

```rust
assert_eq!(TryInto::<i32>::try_into(NumberOrString::String("Hello World".to_owned())), Err(NumberOrString::String("Hello World".to_owned())));
```