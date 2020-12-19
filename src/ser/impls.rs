use super::{Serialize, SerializeError, Serializer};

impl Serialize for i32 {
    fn serialize(&self, serializer: &mut dyn Serializer) -> Result<(), SerializeError> {
        serializer.serialize_signed(*self as i64)?;
        Ok(())
    }
}

impl Serialize for u32 {
    fn serialize(&self, serializer: &mut dyn Serializer) -> Result<(), SerializeError> {
        serializer.serialize_unsigned(*self as u64)?;
        Ok(())
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
