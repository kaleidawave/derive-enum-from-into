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
