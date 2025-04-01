use assert_layout::assert_layout;

// #[assert_layout(size = 9)]
// #[repr(C, packed)]
// struct MyStruct {
//     #[assert_layout(size = 1, offset = 0)]
//     a: u8,
//     #[assert_layout(size = 4, offset = 1)]
//     b: u32,
//     c: u32,
// }
//
// #[assert_layout(size = 9, generics = "u32")]
// #[repr(C, packed)]
// struct MyGenericStruct<T> {
//     a: u8,
//     b: u32,
//     #[assert_layout(size = 4)]
//     c: T,
// }
//
// #[assert_layout(size = 6, generics = "u32, u8")]
// #[repr(C, packed)]
// struct MyDoubleGenericStruct<T, U> {
//     #[assert_layout(offset = 0, size = 1)]
//     a: u8,
//     #[assert_layout(offset = 1, size = 4)]
//     b: T,
//     #[assert_layout(offset = 5, size = 1)]
//     c: U,
// }

#[assert_layout(size = 123, generics = "123")]
#[repr(C, packed)]
struct ConstStruct<const N: usize>([u8; N]);
