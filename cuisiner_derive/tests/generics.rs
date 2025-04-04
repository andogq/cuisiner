use cuisiner::{BigEndian, Cuisiner};

mod primitive {
    use super::*;

    #[derive(Cuisiner)]
    #[cuisiner(assert(size = 1, generics = "u8"))]
    struct Primitive<T: Cuisiner> {
        #[cuisiner(assert(size = 1, offset = 0))]
        value: T,
    }

    #[derive(Cuisiner)]
    #[cuisiner(assert(size = 3, generics = "u8, u16"))]
    struct DoublePrimitive<T: Cuisiner, U: Cuisiner> {
        #[cuisiner(assert(size = 1, offset = 0))]
        value: T,
        #[cuisiner(assert(size = 2, offset = 1))]
        value2: U,
    }

    #[derive(Cuisiner)]
    #[cuisiner(assert(small(generics = "u8", size = 1), big(generics = "u16", size = 2)))]
    struct Namespaced<T: Cuisiner> {
        #[cuisiner(assert(small(offset = 0, size = 1), big(offset = 0, size = 2)))]
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
