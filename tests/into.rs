use derive_enum_from_into::EnumTryInto;
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

    let err: Result<String, _> = NumberOrString::Num(56.0).try_into();
    assert!(err.is_err());
}

#[test]
fn into_reference() {
    #[derive(EnumTryInto, Debug, PartialEq)]
    #[try_into_references(&, ref mut, owned)]
    #[allow(dead_code)]
    enum NumberOrString {
        Num(f32),
        #[try_into_ignore]
        Str(String),
    }

    let number_ref = NumberOrString::Num(4.5);
    assert_eq!((&number_ref).try_into(), Ok(&4.5));
    assert_eq!(TryInto::<&f32>::try_into(&number_ref), Ok(&4.5));
}

#[test]
fn returns_self_if_no_match() {
    #[derive(EnumTryInto, Debug, PartialEq, Clone)]
    #[allow(dead_code)]
    enum NumberOrString {
        Num(f32),
        String(String),
    }

    let number_ref = NumberOrString::Num(4.5);
    assert_eq!(
        TryInto::<String>::try_into(number_ref.clone()),
        Err(number_ref)
    );
}
