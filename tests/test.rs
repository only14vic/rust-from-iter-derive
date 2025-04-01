extern crate alloc;

use {
    alloc::{rc::Rc, sync::Arc},
    core::{
        cell::RefCell,
        error::Error,
        ffi::{c_float, c_uint},
        marker::PhantomData,
        num::NonZero
    },
    set_from_iter_derive::SetFromIter,
    std::time::Instant
};

#[derive(Debug, Default, SetFromIter)]
struct Foo<'a, 'b: 'a, T>
where
    T: AsRef<str>
{
    a: std::string::String,
    b: Option<Box<Option<NonZero<c_uint>>>>,
    c: Arc<bool>,
    d: Option<char>,
    e: Option<Box<Rc<RefCell<f32>>>>,
    f: Option<&'b str>,
    g: Box<Vec<&'a str>>,
    h: Option<String>,
    bar: Bar<'b, T>,
    zar: Zar,
    _phantom: PhantomData<&'b T>
}

#[derive(Debug, Default, SetFromIter)]
struct Bar<'b, T> {
    x: &'b str,
    y: RefCell<c_float>,
    z: Zar,
    _phantom: PhantomData<&'b T>
}

#[derive(Debug, Default, SetFromIter)]
struct Zar {
    a: Option<i32>,
    b: Option<Box<Vec<i32>>>
}

#[test]
fn test_from_map() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        &Foo::<String>::struct_fields()[..2],
        &[
            ("a", "std :: string :: String"),
            ("b", "Option < Box < Option < NonZero < c_uint > > > >"),
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

    let mut foo = Foo::<String>::default();
    foo.h = Some("Predefined value".into());
    foo.zar.b = Some(vec![1, 2, 3].into());

    let t = Instant::now();
    for _ in 0..1 {
        foo.set_from_iter(values.clone())?;
    }
    let time = t.elapsed();
    dbg!(&foo);
    dbg!(time);

    assert_eq!(foo.a, "Hello".to_owned());
    assert_eq!(foo.b, Box::new(NonZero::new(123)).into());
    assert_eq!(foo.c, true.into());
    assert_eq!(foo.d, Some('X'));
    assert_eq!(foo.e, Box::new(Rc::new(RefCell::new(1.23))).into());
    assert_eq!(foo.f, "World".into());
    assert_eq!(foo.g, vec!["a", "b", "c"].into());
    assert_eq!(foo.h, Some("Predefined value".into()));
    assert_eq!(foo.bar.x, "This is Bar");
    assert_eq!(foo.bar.y, 9.999.into());
    assert_eq!(foo.bar.z.a, Some(-1111));
    assert_eq!(foo.bar.z.b, Some(vec![-123, 0, 123].into()));

    assert_eq!(foo.zar.a, Some(-333));
    assert_eq!(foo.zar.b, Some(vec![1, 2, 3].into()));

    Ok(())
}
