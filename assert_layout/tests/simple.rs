use assert_layout::assert_layout;

#[assert_layout(size = 9)]
#[repr(C, packed)]
struct MyStruct {
    #[assert_layout(size = 1, offset = 0)]
    a: u8,
    #[assert_layout(size = 4, offset = 1)]
    b: u32,
    c: u32,
}

#[assert_layout(size = 9, generics = "u32", generics = "i32")]
#[repr(C, packed)]
struct MyGenericStruct<T> {
    a: u8,
    b: u32,
    #[assert_layout(size = 4)]
    c: T,
}

#[assert_layout(size = 6, generics = "u32, u8")]
#[repr(C, packed)]
struct MyDoubleGenericStruct<T, U> {
    #[assert_layout(offset = 0, size = 1)]
    a: u8,
    #[assert_layout(offset = 1, size = 4)]
    b: T,
    #[assert_layout(offset = 5, size = 1)]
    c: U,
}

#[assert_layout(size = 123, generics = "123")]
#[repr(C, packed)]
struct ConstStruct<const N: usize>(#[assert_layout(size = 123)] [u8; N]);

#[assert_layout(generics = "u16")]
#[repr(C, packed)]
struct GenericWithTrait<T: TheTrait> {
    #[assert_layout(offset = 0, size = 2)]
    item: T,
    #[assert_layout(offset = 2, size = 4)]
    nested: T::Item,
}

trait TheTrait {
    type Item;
}

impl TheTrait for u16 {
    type Item = u32;
}

#[assert_layout(generics = "u64")]
#[repr(C, packed)]
struct GenericWithNestedTrait<T: NestedTrait> {
    #[assert_layout(offset = 0, size = 8)]
    item: T,
    #[assert_layout(offset = 8, size = 2)]
    nested: T::Nested,
    #[assert_layout(offset = 10, size = 4)]
    deep_nested: <T::Nested as TheTrait>::Item,
}

trait NestedTrait {
    type Nested: TheTrait;
}

impl NestedTrait for u64 {
    type Nested = u16;
}

#[assert_layout(generics = "u8", size = 5, big(generics = "u16", size = 6))]
#[repr(C, packed)]
struct NamespacedStruct<T> {
    #[assert_layout(offset = 0, size = 1, big(offset = 0, size = 2))]
    thing: T,

    #[assert_layout(offset = 1, size = 4, big(offset = 2, size = 4))]
    another: u32,
}

#[assert_layout(
    generics = "u8",
    generics = "u16",
    little(generics = "u8", size = 5),
    big(generics = "u16", size = 6)
)]
#[repr(C, packed)]
struct NamespacedStruct2<T> {
    #[assert_layout(offset = 0, little(size = 1), big(size = 2))]
    thing: T,

    #[assert_layout(size = 4, little(offset = 1), big(offset = 2))]
    another: u32,
}
