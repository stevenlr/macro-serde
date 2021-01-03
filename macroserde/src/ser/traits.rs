#[derive(Debug)]
pub struct SerializeError;

pub trait Serializer {
    fn serialize_null(&mut self) -> Result<(), SerializeError>;
    fn serialize_bool(&mut self, value: bool) -> Result<(), SerializeError>;
    fn serialize_signed(&mut self, value: i64) -> Result<(), SerializeError>;
    fn serialize_unsigned(&mut self, value: u64) -> Result<(), SerializeError>;
    fn serialize_float(&mut self, value: f64) -> Result<(), SerializeError>;
    fn serialize_str(&mut self, value: &str) -> Result<(), SerializeError>;
    fn serialize_enum(&mut self, value: u32, name: &'static str) -> Result<(), SerializeError>;
    fn start_struct(&mut self, len: usize) -> Result<(), SerializeError>;
    fn serialize_struct_field(
        &mut self,
        field_id: u32,
        field_name: &'static str,
        value: &dyn Serialize,
    ) -> Result<(), SerializeError>;
    fn end_struct(&mut self) -> Result<(), SerializeError>;
    fn start_seq(&mut self, len: usize) -> Result<(), SerializeError>;
    fn serialize_seq_elmt(&mut self, value: &dyn Serialize) -> Result<(), SerializeError>;
    fn end_seq(&mut self) -> Result<(), SerializeError>;
}

pub trait Serialize {
    fn serialize(&self, serializer: &mut dyn Serializer) -> Result<(), SerializeError>;
}
