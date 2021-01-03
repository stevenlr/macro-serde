use json;
use macroserde::{de, ser};
use std::io;

pub struct Serializer {
    current_value: json::JsonValue,
    stack: Vec<json::JsonValue>,
}

impl Serializer {
    pub fn new() -> Self {
        Self {
            current_value: json::JsonValue::Null,
            stack: Vec::new(),
        }
    }

    pub fn write_pretty<W: io::Write>(&self, writer: &mut W, spaces: u16) -> io::Result<()> {
        self.current_value.write_pretty(writer, spaces)
    }

    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        self.current_value.write(writer)
    }
}

impl ser::Serializer for Serializer {
    fn serialize_null(&mut self) -> Result<(), ser::SerializeError> {
        self.current_value = json::JsonValue::Null;
        Ok(())
    }

    fn serialize_bool(&mut self, value: bool) -> Result<(), ser::SerializeError> {
        self.current_value = json::JsonValue::Boolean(value);
        Ok(())
    }

    fn serialize_signed(&mut self, value: i64) -> Result<(), ser::SerializeError> {
        self.current_value = json::JsonValue::from(value);
        Ok(())
    }

    fn serialize_unsigned(&mut self, value: u64) -> Result<(), ser::SerializeError> {
        self.current_value = json::JsonValue::from(value);
        Ok(())
    }

    fn serialize_float(&mut self, value: f64) -> Result<(), ser::SerializeError> {
        self.current_value = json::JsonValue::from(value);
        Ok(())
    }

    fn serialize_str(&mut self, value: &str) -> Result<(), ser::SerializeError> {
        self.current_value = json::JsonValue::from(value);
        Ok(())
    }

    fn serialize_enum(
        &mut self,
        value: u32,
        name: &'static str,
    ) -> Result<(), ser::SerializeError> {
        self.current_value = json::JsonValue::from(format!("{}:{}", value, name));
        Ok(())
    }

    fn start_struct(&mut self, _len: usize) -> Result<(), ser::SerializeError> {
        self.stack.push(json::JsonValue::new_object());
        Ok(())
    }

    fn serialize_struct_field(
        &mut self,
        field_id: u32,
        field_name: &'static str,
        value: &dyn ser::Serialize,
    ) -> Result<(), ser::SerializeError> {
        value.serialize(self)?;
        if let Some(json::JsonValue::Object(obj)) = self.stack.last_mut() {
            obj.insert(
                &format!("{}:{}", field_id, field_name),
                std::mem::replace(&mut self.current_value, json::JsonValue::Null),
            );
        }
        Ok(())
    }

    fn end_struct(&mut self) -> Result<(), ser::SerializeError> {
        self.current_value = self.stack.pop().unwrap();
        Ok(())
    }

    fn start_seq(&mut self, _len: usize) -> Result<(), ser::SerializeError> {
        self.stack.push(json::JsonValue::new_array());
        Ok(())
    }

    fn serialize_seq_elmt(
        &mut self,
        value: &dyn ser::Serialize,
    ) -> Result<(), ser::SerializeError> {
        value.serialize(self)?;
        if let Some(json::JsonValue::Array(array)) = self.stack.last_mut() {
            array.push(std::mem::replace(
                &mut self.current_value,
                json::JsonValue::Null,
            ));
        }
        Ok(())
    }

    fn end_seq(&mut self) -> Result<(), ser::SerializeError> {
        self.current_value = self.stack.pop().unwrap();
        Ok(())
    }
}

pub struct Deserializer {
    value: json::JsonValue,
}

impl Deserializer {
    pub fn new(s: &str) -> Option<Self> {
        Some(Self {
            value: json::parse(s).ok()?,
        })
    }

    fn split_key(s: &str) -> (Option<u32>, &str) {
        if let Some(sep) = s.find(':') {
            if let Ok(id) = s[..sep].parse::<u32>() {
                return (Some(id), &s[(sep + 1)..]);
            }
        }
        return (None, s);
    }

    fn visit_value(
        value: &json::JsonValue,
        visitor: &mut dyn de::Visitor,
    ) -> Result<(), de::DeserializeError> {
        match value {
            json::JsonValue::Null => visitor.visit_null(),
            json::JsonValue::Short(s) => visitor.visit_str(s.as_str()),
            json::JsonValue::String(s) => visitor.visit_str(s.as_str()),
            json::JsonValue::Number(_) => Self::visit_number(value, visitor),
            json::JsonValue::Boolean(val) => visitor.visit_bool(*val),
            json::JsonValue::Object(_) => Self::visit_object(value, visitor),
            json::JsonValue::Array(_) => Self::visit_array(value, visitor),
        }
    }

    fn visit_array(
        value: &json::JsonValue,
        visitor: &mut dyn de::Visitor,
    ) -> Result<(), de::DeserializeError> {
        let mut builder = visitor.visit_seq(Some(value.len()))?;
        for entry in value.members() {
            let visitor = builder.element()?;
            Self::visit_value(entry, visitor)?;
        }
        builder.finish()
    }

    fn visit_object(
        value: &json::JsonValue,
        visitor: &mut dyn de::Visitor,
    ) -> Result<(), de::DeserializeError> {
        let mut builder = visitor.visit_struct()?;
        for entry in value.entries() {
            let (id, name) = Self::split_key(entry.0);
            let visitor = builder.member(id, Some(name))?;
            Self::visit_value(entry.1, visitor)?;
        }
        builder.finish()
    }

    fn visit_number(
        number: &json::JsonValue,
        visitor: &mut dyn de::Visitor,
    ) -> Result<(), de::DeserializeError> {
        if let Some(u) = number.as_u64() {
            visitor.visit_unsigned(u)
        } else if let Some(i) = number.as_i64() {
            visitor.visit_signed(i)
        } else if let Some(f) = number.as_f64() {
            visitor.visit_float(f)
        } else {
            Err(de::DeserializeError::IncompatibleNumericType)
        }
    }
}

impl de::Deserializer for Deserializer {
    fn deserialize(&mut self, visitor: &mut dyn de::Visitor) -> Result<(), de::DeserializeError> {
        Self::visit_value(&self.value, visitor)
    }
}
