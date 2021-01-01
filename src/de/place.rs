#[macro_export]
macro_rules! make_place_type {
    ($vis:vis $name:ident) => {
        $vis struct $name<T> {
            $vis out: Option<T>,
        }

        impl<T> $name<T> {
            $vis fn new(out: &mut Option<T>) -> &mut Self {
                unsafe { &mut *{ out as *mut Option<T> as *mut Self } }
            }
        }
    }
}

make_place_type!(pub(crate) Place);
