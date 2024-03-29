use derive_enum_from_into::EnumFrom;

#[test]
fn from() {
    #[derive(EnumFrom, Debug, PartialEq)]
    enum NumberOrString {
        Num(f32),
        Str(String),
    }

    assert_eq!(
        NumberOrString::from("Hi".to_owned()),
        NumberOrString::Str("Hi".to_owned())
    );
    assert_eq!(NumberOrString::from(5.1), NumberOrString::Num(5.1));
}

#[test]
fn from_with_generics() {
    #[derive(EnumFrom, Debug, PartialEq)]
    enum StringOrNumberRef<'a> {
        Str(&'a mut String),
        Num(&'a f32),
    }

    assert_eq!(
        StringOrNumberRef::from(&mut "Hi".to_owned()),
        StringOrNumberRef::Str(&mut "Hi".to_owned())
    );
    assert_eq!(StringOrNumberRef::from(&5.1), StringOrNumberRef::Num(&5.1));
}
