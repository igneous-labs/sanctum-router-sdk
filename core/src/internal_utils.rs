#![macro_use]

/// Example-usage:
///
/// ```ignore
/// seqconsts!(ty = usize; count = COUNT; A, B);
/// ```
///
/// generates:
///
/// ```
/// pub const A: usize = 0;
/// pub const B: usize = 1;
/// pub const COUNT: usize = 2;
/// ```
macro_rules! seqconsts {
    // recursive-case
    (@cnt $cnt:expr; ty = $type:ty; count = $count_name:ident; $name:ident $(, $($tail:tt)*)?) => {
        pub const $name: $type = $cnt;
        seqconsts!(@cnt ($cnt + 1); ty = $type; count = $count_name; $($($tail)*)?);
    };

    // base-cases
    (@cnt $cnt:expr; ty = $type:ty; count = $count_name:ident;) => {
        pub const $count_name: $type = $cnt;
    };
    () => {};

    // start
    ($($tail:tt)*) => { seqconsts!(@cnt 0; $($tail)*); };
}
pub(crate) use seqconsts;
