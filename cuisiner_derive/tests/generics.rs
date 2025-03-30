use cuisiner::{BigEndian, Cuisiner};

mod primitive {
    use super::*;

    #[derive(Cuisiner)]
    #[cuisiner(assert_size = 2, assert_generics = "i16", assert_generics = "u16")]
    struct Primitive<T: Cuisiner> {
        value: T,
    }
}

mod random {
    use super::*;

    #[derive(Cuisiner, Debug, PartialEq, Eq)]
    struct MyStruct<T: Cuisiner> {
        value: u32,
        nested: T,
    }

    #[derive(Cuisiner, Debug, PartialEq, Eq)]
    struct InnerU8 {
        value: u8,
    }

    #[derive(Cuisiner, Debug, PartialEq, Eq)]
    struct InnerU64 {
        value: u64,
    }

    #[test]
    fn generic_small() {
        let bytes = MyStruct {
            value: 1234,
            nested: InnerU8 { value: 0xff },
        }
        .to_bytes::<BigEndian>()
        .unwrap();

        assert_eq!(
            MyStruct {
                value: 1234,
                nested: InnerU8 { value: 0xff },
            },
            MyStruct::from_bytes::<BigEndian>(&bytes).unwrap()
        );
    }

    #[test]
    fn generic_big() {
        let bytes = MyStruct {
            value: 1234,
            nested: InnerU64 {
                value: 0xfedcba0987654321,
            },
        }
        .to_bytes::<BigEndian>()
        .unwrap();

        assert_eq!(
            MyStruct {
                value: 1234,
                nested: InnerU64 {
                    value: 0xfedcba0987654321,
                },
            },
            MyStruct::from_bytes::<BigEndian>(&bytes).unwrap()
        );
    }
}
