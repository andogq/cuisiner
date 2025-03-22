use cuisiner::Cuisiner;

#[derive(Clone, Cuisiner, Debug, PartialEq, Eq)]
struct MyStruct {
    a: i32,
    b: u64,
    s: S2,
}

#[derive(Clone, Cuisiner, Debug, PartialEq, Eq)]
struct S2 {
    thing: u64,
}

#[test]
fn deserialse() {
    let s = MyStruct {
        a: -12,
        b: 1234,
        s: S2 { thing: 4321 },
    };

    let b = s.clone().to_bytes();
    dbg!(&b);

    let s2 = MyStruct::from_bytes(&b);
    dbg!(&s2);

    assert_eq!(s, s2);
}
