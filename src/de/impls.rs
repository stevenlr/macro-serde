use super::{Deserialize, DeserializeError, SeqBuilder, Visitor};
use crate::make_place_type;

make_place_type!(Place);

macro_rules! deserialize_signed {
    ($type:ty, $min:path, $max:path) => {
        impl Deserialize for $type {
            fn begin_deserialize(out: &mut Option<Self>) -> &mut dyn Visitor {
                impl Visitor for Place<$type> {
                    fn visit_signed(&mut self, value: i64) -> Result<(), DeserializeError> {
                        if value < $min as i64 || value > $max as i64 {
                            return Err(DeserializeError::IncompatibleNumericType);
                        } else {
                            self.out.replace(value as $type);
                            Ok(())
                        }
                    }

                    fn visit_unsigned(&mut self, value: u64) -> Result<(), DeserializeError> {
                        if value > $max as u64 {
                            return Err(DeserializeError::IncompatibleNumericType);
                        } else {
                            self.out.replace(value as $type);
                            Ok(())
                        }
                    }

                    fn visit_float(&mut self, value: f64) -> Result<(), DeserializeError> {
                        if value < $min as f64 || value > $max as f64 {
                            return Err(DeserializeError::IncompatibleNumericType);
                        } else {
                            self.out.replace(value as $type);
                            Ok(())
                        }
                    }

                    fn visit_bool(&mut self, value: bool) -> Result<(), DeserializeError> {
                        self.out.replace(if value { 1 } else { 0 });
                        Ok(())
                    }
                }
                return Place::new(out);
            }
        }
    };
}

deserialize_signed!(i8, std::i8::MIN, std::i8::MAX);
deserialize_signed!(i16, std::i16::MIN, std::i16::MAX);
deserialize_signed!(i32, std::i32::MIN, std::i32::MAX);
deserialize_signed!(i64, std::i64::MIN, std::i64::MAX);
deserialize_signed!(isize, std::isize::MIN, std::isize::MAX);

macro_rules! deserialize_unsigned {
    ($type:ty, $max:path) => {
        impl Deserialize for $type {
            fn begin_deserialize(out: &mut Option<Self>) -> &mut dyn Visitor {
                impl Visitor for Place<$type> {
                    fn visit_signed(&mut self, value: i64) -> Result<(), DeserializeError> {
                        if value < 0 || value > $max as i64 {
                            return Err(DeserializeError::IncompatibleNumericType);
                        } else {
                            self.out.replace(value as $type);
                            Ok(())
                        }
                    }

                    fn visit_unsigned(&mut self, value: u64) -> Result<(), DeserializeError> {
                        if value > $max as u64 {
                            return Err(DeserializeError::IncompatibleNumericType);
                        } else {
                            self.out.replace(value as $type);
                            Ok(())
                        }
                    }

                    fn visit_float(&mut self, value: f64) -> Result<(), DeserializeError> {
                        if value < 0.0 || value > $max as f64 {
                            return Err(DeserializeError::IncompatibleNumericType);
                        } else {
                            self.out.replace(value as $type);
                            Ok(())
                        }
                    }

                    fn visit_bool(&mut self, value: bool) -> Result<(), DeserializeError> {
                        self.out.replace(if value { 1 } else { 0 });
                        Ok(())
                    }
                }
                return Place::new(out);
            }
        }
    };
}

deserialize_unsigned!(u8, std::u8::MAX);
deserialize_unsigned!(u16, std::u16::MAX);
deserialize_unsigned!(u32, std::u32::MAX);
deserialize_unsigned!(u64, std::u64::MAX);
deserialize_unsigned!(usize, std::usize::MAX);

macro_rules! deserialize_float {
    ($type:ty) => {
        impl Deserialize for $type {
            fn begin_deserialize(out: &mut Option<Self>) -> &mut dyn Visitor {
                impl Visitor for Place<$type> {
                    fn visit_signed(&mut self, value: i64) -> Result<(), DeserializeError> {
                        self.out.replace(value as $type);
                        Ok(())
                    }

                    fn visit_unsigned(&mut self, value: u64) -> Result<(), DeserializeError> {
                        self.out.replace(value as $type);
                        Ok(())
                    }

                    fn visit_float(&mut self, value: f64) -> Result<(), DeserializeError> {
                        self.out.replace(value as $type);
                        Ok(())
                    }

                    fn visit_bool(&mut self, value: bool) -> Result<(), DeserializeError> {
                        self.out.replace(if value { 1.0 } else { 0.0 });
                        Ok(())
                    }
                }
                return Place::new(out);
            }
        }
    };
}

deserialize_float!(f32);
deserialize_float!(f64);

impl<T: Deserialize> Deserialize for Option<T> {
    fn begin_deserialize(out: &mut Option<Self>) -> &mut dyn Visitor {
        impl<T: Deserialize> Visitor for Place<Option<T>> {
            fn visit_null(&mut self) -> Result<(), DeserializeError> {
                self.out.replace(None);
                Ok(())
            }

            fn visit_bool(&mut self, value: bool) -> Result<(), DeserializeError> {
                let mut place = None;
                T::begin_deserialize(&mut place).visit_bool(value)?;
                self.out.replace(place);
                Ok(())
            }

            fn visit_signed(&mut self, value: i64) -> Result<(), DeserializeError> {
                let mut place = None;
                T::begin_deserialize(&mut place).visit_signed(value)?;
                self.out.replace(place);
                Ok(())
            }

            fn visit_unsigned(&mut self, value: u64) -> Result<(), DeserializeError> {
                let mut place = None;
                T::begin_deserialize(&mut place).visit_unsigned(value)?;
                self.out.replace(place);
                Ok(())
            }

            fn visit_float(&mut self, value: f64) -> Result<(), DeserializeError> {
                let mut place = None;
                T::begin_deserialize(&mut place).visit_float(value)?;
                self.out.replace(place);
                Ok(())
            }

            fn visit_str(&mut self, value: &str) -> Result<(), DeserializeError> {
                let mut place = None;
                T::begin_deserialize(&mut place).visit_str(value)?;
                self.out.replace(place);
                Ok(())
            }
        }

        return Place::new(out);
    }
}

impl Deserialize for bool {
    fn begin_deserialize(out: &mut Option<Self>) -> &mut dyn Visitor {
        impl Visitor for Place<bool> {
            fn visit_signed(&mut self, value: i64) -> Result<(), DeserializeError> {
                self.out.replace(value != 0);
                Ok(())
            }

            fn visit_unsigned(&mut self, value: u64) -> Result<(), DeserializeError> {
                self.out.replace(value != 0);
                Ok(())
            }

            fn visit_float(&mut self, value: f64) -> Result<(), DeserializeError> {
                self.out.replace(value != 0.0);
                Ok(())
            }

            fn visit_bool(&mut self, value: bool) -> Result<(), DeserializeError> {
                self.out.replace(value);
                Ok(())
            }
        }
        return Place::new(out);
    }
}

impl Deserialize for String {
    fn begin_deserialize(out: &mut Option<Self>) -> &mut dyn Visitor {
        impl Visitor for Place<String> {
            fn visit_str(&mut self, value: &str) -> Result<(), DeserializeError> {
                self.out.replace(value.to_owned());
                Ok(())
            }
        }
        return Place::new(out);
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn begin_deserialize(out: &mut Option<Self>) -> &mut dyn Visitor {
        struct Builder<'a, T> {
            out: &'a mut Option<Vec<T>>,
            vec: Vec<T>,
            elmt: Option<T>,
        }

        impl<'a, T> Builder<'a, T> {
            fn new(out: &'a mut Option<Vec<T>>) -> Self {
                Self {
                    out,
                    vec: Vec::new(),
                    elmt: None,
                }
            }

            fn shift(&mut self) {
                if let Some(e) = self.elmt.take() {
                    self.vec.push(e);
                }
            }
        }

        impl<'a, T: Deserialize> SeqBuilder for Builder<'a, T> {
            fn element(&mut self) -> Result<&mut dyn Visitor, DeserializeError> {
                self.shift();
                Ok(T::begin_deserialize(&mut self.elmt))
            }

            fn finish(&mut self) -> Result<(), DeserializeError> {
                self.shift();
                self.out
                    .replace(std::mem::replace(&mut self.vec, Vec::new()));
                Ok(())
            }
        }

        impl<T: Deserialize> Visitor for Place<Vec<T>> {
            fn visit_seq(
                &mut self,
                _size_hint: Option<usize>,
            ) -> Result<Box<dyn SeqBuilder + '_>, DeserializeError> {
                Ok(Box::new(Builder::new(&mut self.out)))
            }
        }
        return Place::new(out);
    }
}

impl Deserialize for () {
    fn begin_deserialize(out: &mut Option<Self>) -> &mut dyn Visitor {
        impl Visitor for Place<()> {
            fn visit_null(&mut self) -> Result<(), DeserializeError> {
                self.out.replace(());
                Ok(())
            }
        }
        return Place::new(out);
    }
}
