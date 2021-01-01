#[macro_export]
macro_rules! make_place_type {
    ($vis:vis $name:ident) => {
        #[allow(unused)]
        $vis struct $name<T> {
            $vis out: Option<T>,
        }

        impl<T> $name<T> {
            #[allow(unused)]
            $vis fn new(out: &mut Option<T>) -> &mut Self {
                unsafe { &mut *{ out as *mut Option<T> as *mut Self } }
            }
        }
    }
}
