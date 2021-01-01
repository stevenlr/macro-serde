use super::{Serialize, SerializeError, Serializer};

macro_rules! serialize_signed {
    ($ty:ty) => {
        impl Serialize for $ty {
            fn serialize(&self, serializer: &mut dyn Serializer) -> Result<(), SerializeError> {
                serializer.serialize_signed(*self as i64)?;
                Ok(())
            }
        }
    };
}

serialize_signed!(i8);
serialize_signed!(i16);
serialize_signed!(i32);
serialize_signed!(i64);
serialize_signed!(isize);

macro_rules! serialize_unsigned {
    ($ty:ty) => {
        impl Serialize for $ty {
            fn serialize(&self, serializer: &mut dyn Serializer) -> Result<(), SerializeError> {
                serializer.serialize_unsigned(*self as u64)?;
                Ok(())
            }
        }
    };
}

serialize_unsigned!(u8);
serialize_unsigned!(u16);
serialize_unsigned!(u32);
serialize_unsigned!(u64);
serialize_unsigned!(usize);

macro_rules! serialize_float {
    ($ty:ty) => {
        impl Serialize for $ty {
            fn serialize(&self, serializer: &mut dyn Serializer) -> Result<(), SerializeError> {
                serializer.serialize_float(*self as f64)?;
                Ok(())
            }
        }
    };
}

serialize_float!(f32);
serialize_float!(f64);

impl Serialize for bool {
    fn serialize(&self, serializer: &mut dyn Serializer) -> Result<(), SerializeError> {
        serializer.serialize_bool(*self)?;
        Ok(())
    }
}

impl<T: Serialize> Serialize for Option<T> {
    fn serialize(&self, serializer: &mut dyn Serializer) -> Result<(), SerializeError> {
        match self {
            Some(s) => s.serialize(serializer),
            None => serializer.serialize_null(),
        }
    }
}

impl Serialize for String {
    fn serialize(&self, serializer: &mut dyn Serializer) -> Result<(), SerializeError> {
        serializer.serialize_str(self.as_ref())?;
        Ok(())
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize(&self, serializer: &mut dyn Serializer) -> Result<(), SerializeError> {
        serializer.start_seq(self.len())?;
        for elmt in self.iter() {
            serializer.serialize_seq_elmt(elmt)?;
        }
        serializer.end_seq()?;
        Ok(())
    }
}

impl From<std::fmt::Error> for SerializeError {
    fn from(_: std::fmt::Error) -> SerializeError {
        SerializeError {}
    }
}

impl From<std::io::Error> for SerializeError {
    fn from(_: std::io::Error) -> SerializeError {
        SerializeError {}
    }
}
