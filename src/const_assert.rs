pub struct True;

pub struct False;

pub trait ToBool {
    type BoolType;
    const TO_BOOL: Self::BoolType;
}

impl ToBool for [(); 0] {
    type BoolType = False;
    const TO_BOOL: Self::BoolType = False;
}

impl ToBool for [(); 1] {
    type BoolType = True;
    const TO_BOOL: Self::BoolType = True;
}

#[macro_export]
macro_rules! to_bool {
    ($x:expr) => {{
        const B: bool = $x;
        <[(); B as usize] as $crate::const_assert::ToBool>::TO_BOOL
    }};
}

#[macro_export]
macro_rules! const_assert {
    ($x:expr) => {
        const _: $crate::const_assert::True = to_bool!($x);
    };
}
