use derive_enum_from_try_into::EnumTryInto;
use std::convert::TryInto;

#[test]
fn into() {
    #[derive(EnumTryInto, Debug, PartialEq)]
    enum NumberOrString {
        Num(f32),
        Str(String),
    }

    assert_eq!(NumberOrString::Num(4.2).try_into(), Ok(4.2));
    assert_eq!(
        NumberOrString::Str("Hello".to_owned()).try_into(),
        Ok("Hello".to_owned())
    );

    let err: Result<String, ()> = NumberOrString::Num(56.0).try_into();
    assert_eq!(err.unwrap_err(), ());
}

#[test]
fn into_reference() {
    #[derive(EnumTryInto, Debug, PartialEq)]
    #[try_into_references(&, ref mut)]
    #[allow(dead_code)]
    enum NumberOrString {
        Num(f32),
        #[try_into_ignore]
        Str(String),
    }

    let number_ref = NumberOrString::Num(4.5);

    assert_eq!((&number_ref).try_into(), Ok(&4.5));
}
