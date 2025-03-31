extern crate alloc;

use {from_map_derive::FromMap, std::collections::BTreeMap};

#[derive(Debug, Default, FromMap)]
struct Foo {
    a: String,
    b: Option<u32>,
    c: bool,
    d: Option<char>,
    e: f32,
    f: Box<str>,
    g: Box<[char]>,
    h: Option<String>
}

#[derive(Debug, Default, FromMap)]
struct Bar {
    x: Box<str>,
    y: f32
}

#[test]
fn test_from_map() {
    for (name, ty) in Foo::struct_fields() {
        println!("{name}: {ty}");
    }

    let values: Vec<(&str, Option<&str>)> = vec![
        ("a", "   Hello  ".into()),
        ("b", "  123  ".into()),
        ("c", "   true  ".into()),
        ("d", "    X   ".into()),
        ("e", "  1.23  ".into()),
        ("f", "  World  ".into()),
        ("g", "Yes".into()),
        ("h", None),
    ];

    let mut map = BTreeMap::from_iter(values.clone().into_iter());
    map.remove("a");

    let foo = Foo::from_iter(values);
    dbg!(foo);
}
