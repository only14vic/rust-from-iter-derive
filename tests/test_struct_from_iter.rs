extern crate alloc;

use {
    core::{
        cell::RefCell,
        ffi::{c_float, c_uint},
        num::NonZero
    },
    from_iter_derive::FromIter,
    std::{rc::Rc, sync::Arc}
};

#[derive(Debug, Default, FromIter)]
struct Foo {
    a: String,
    b: Option<Box<Option<NonZero<c_uint>>>>,
    c: Arc<bool>,
    d: Option<char>,
    e: Option<Box<Rc<RefCell<f32>>>>,
    f: Box<str>,
    g: Vec<Box<str>>,
    h: Option<String>,
    bar: Bar,
    zar: Option<Box<Zar>>
}

#[derive(Debug, Default, FromIter)]
struct Bar {
    x: Box<str>,
    y: RefCell<c_float>,
    z: Zar
}

#[derive(Debug, Default, FromIter)]
struct Zar {
    a: Option<i32>,
    b: Option<Vec<i32>>
}

#[test]
fn test_from_map() {
    assert_eq!(
        Foo::struct_fields().collect::<Vec<(&str, &str)>>(),
        vec![
            ("a", "String"),
            ("b", "Option < Box < Option < NonZero < c_uint > > > >"),
            ("c", "Arc < bool >"),
            ("d", "Option < char >"),
            ("e", "Option < Box < Rc < RefCell < f32 > > > >"),
            ("f", "Box < str >"),
            ("g", "Vec < Box < str > >"),
            ("h", "Option < String >"),
            ("bar", "Bar"),
            ("zar", "Option < Box < Zar > >")
        ]
    );

    let values: Vec<(&str, Option<&str>)> = vec![
        ("a", "   Hello  ".into()),
        ("b", "  123  ".into()),
        ("c", "   true  ".into()),
        ("d", "    X   ".into()),
        ("e", "  1.23  ".into()),
        ("f", "  World  ".into()),
        ("g", "a , b , c ".into()),
        ("h", None),
        ("bar.x", "This is Bar".into()),
        ("bar.y", "  9.999".into()),
        ("bar.z.a", "  -1111 ".into()),
        ("bar.z.b", "  -123, 0, 123 ".into()),
        ("zar.a", " -333 ".into()),
    ];

    let foo = Foo::from_iter(values);
    dbg!(&foo);

    assert_eq!(foo.a, "Hello".to_owned());
    assert_eq!(foo.b, Box::new(NonZero::new(123)).into());
    assert_eq!(foo.c, true.into());
    assert_eq!(foo.d, Some('X'));
    assert_eq!(foo.e, Box::new(Rc::new(RefCell::new(1.23))).into());
    assert_eq!(foo.g, vec!["a".into(), "b".into(), "c".into()]);
    assert_eq!(foo.h, None);
    assert_eq!(foo.bar.x, "This is Bar".into());
    assert_eq!(foo.bar.y, 9.999.into());
    assert_eq!(foo.bar.z.a, Some(-1111));
    assert_eq!(foo.bar.z.b, Some(vec![-123, 0, 123]));

    assert!(foo.zar.is_some());
    assert_eq!(foo.zar.as_ref().unwrap().a, Some(-333));
    assert_eq!(foo.zar.as_ref().unwrap().b, None);
}
