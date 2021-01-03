use macroserde::ser;
use std::io;
use std::io::Write;

pub struct Serializer<W: io::Write> {
    write: io::BufWriter<W>,
}

impl<W: io::Write> Serializer<W> {
    pub fn new(w: W) -> Self {
        Self {
            write: io::BufWriter::new(w),
        }
    }

    fn write_unsigned_8_to_64(
        &mut self,
        discriminant: u8,
        value: u64,
    ) -> Result<(), ser::SerializeError> {
        if value <= std::u8::MAX as u64 {
            self.write.write_all(&[discriminant])?;
            self.write.write_all(&(value as u8).to_be_bytes())?;
        } else if value <= std::u16::MAX as u64 {
            self.write.write_all(&[discriminant + 1])?;
            self.write.write_all(&(value as u16).to_be_bytes())?;
        } else if value <= std::u32::MAX as u64 {
            self.write.write_all(&[discriminant + 2])?;
            self.write.write_all(&(value as u32).to_be_bytes())?;
        } else {
            self.write.write_all(&[discriminant + 3])?;
            self.write.write_all(&value.to_be_bytes())?;
        }
        Ok(())
    }

    fn write_unsigned_16_to_32(
        &mut self,
        discriminant: u8,
        value: u64,
    ) -> Result<(), ser::SerializeError> {
        if value <= std::u16::MAX as u64 {
            self.write.write_all(&[discriminant])?;
            self.write.write_all(&(value as u16).to_be_bytes())?;
            Ok(())
        } else if value <= std::u32::MAX as u64 {
            self.write.write_all(&[discriminant + 1])?;
            self.write.write_all(&(value as u32).to_be_bytes())?;
            Ok(())
        } else {
            Err(ser::SerializeError)
        }
    }
}

impl<W: io::Write> Drop for Serializer<W> {
    fn drop(&mut self) {
        self.write.flush().unwrap();
    }
}

impl<W: io::Write> ser::Serializer for Serializer<W> {
    fn serialize_null(&mut self) -> Result<(), ser::SerializeError> {
        self.write.write_all(&[0xc0])?;
        Ok(())
    }

    fn serialize_bool(&mut self, value: bool) -> Result<(), ser::SerializeError> {
        self.write.write_all(&[0xc2 + value as u8])?;
        Ok(())
    }

    fn serialize_signed(&mut self, value: i64) -> Result<(), ser::SerializeError> {
        if value >= -32 && value <= 127 {
            self.write.write_all(&(value as i8).to_be_bytes())?;
        } else if value >= std::i8::MIN as i64 && value <= std::i8::MAX as i64 {
            self.write.write_all(&[0xd0])?;
            self.write.write_all(&(value as i8).to_be_bytes())?;
        } else if value >= std::i16::MIN as i64 && value <= std::i16::MAX as i64 {
            self.write.write_all(&[0xd1])?;
            self.write.write_all(&(value as i16).to_be_bytes())?;
        } else if value >= std::i32::MIN as i64 && value <= std::i32::MAX as i64 {
            self.write.write_all(&[0xd2])?;
            self.write.write_all(&(value as i32).to_be_bytes())?;
        } else {
            self.write.write_all(&[0xd3])?;
            self.write.write_all(&value.to_be_bytes())?;
        }
        Ok(())
    }

    fn serialize_unsigned(&mut self, value: u64) -> Result<(), ser::SerializeError> {
        if value <= 127 {
            self.write.write_all(&(value as u8).to_be_bytes())?;
        } else {
            self.write_unsigned_8_to_64(0xcc, value)?;
        }
        Ok(())
    }

    fn serialize_float(&mut self, value: f64) -> Result<(), ser::SerializeError> {
        if value as f32 as f64 == value {
            self.write.write_all(&[0xca])?;
            self.write.write_all(&(value as f32).to_be_bytes())?;
        } else {
            self.write.write_all(&[0xcb])?;
            self.write.write_all(&value.to_be_bytes())?;
        }
        Ok(())
    }

    fn serialize_str(&mut self, value: &str) -> Result<(), ser::SerializeError> {
        if value.len() > std::u32::MAX as usize {
            return Err(ser::SerializeError);
        } else if value.len() < 32 {
            self.write.write_all(&[0xa0 + value.len() as u8])?;
        } else {
            self.write_unsigned_8_to_64(0xd9, value.len() as u64)?;
        }
        self.write.write_all(value.as_bytes())?;
        Ok(())
    }

    fn serialize_enum(
        &mut self,
        value: u32,
        _name: &'static str,
    ) -> Result<(), ser::SerializeError> {
        self.serialize_unsigned(value as u64)
    }

    fn start_struct(&mut self, len: usize) -> Result<(), ser::SerializeError> {
        if len < 16 {
            self.write.write_all(&[0x80 + len as u8])?;
        } else {
            self.write_unsigned_16_to_32(0xde, len as u64)?;
        }
        Ok(())
    }

    fn serialize_struct_field(
        &mut self,
        field_id: u32,
        _field_name: &'static str,
        value: &dyn ser::Serialize,
    ) -> Result<(), ser::SerializeError> {
        self.serialize_unsigned(field_id as u64)?;
        value.serialize(self)
    }

    fn end_struct(&mut self) -> Result<(), ser::SerializeError> {
        Ok(())
    }

    fn start_seq(&mut self, len: usize) -> Result<(), ser::SerializeError> {
        if len < 16 {
            self.write.write_all(&[0x90 + len as u8])?;
        } else {
            self.write_unsigned_16_to_32(0xdc, len as u64)?;
        }
        Ok(())
    }

    fn serialize_seq_elmt(
        &mut self,
        value: &dyn ser::Serialize,
    ) -> Result<(), ser::SerializeError> {
        value.serialize(self)
    }

    fn end_seq(&mut self) -> Result<(), ser::SerializeError> {
        Ok(())
    }
}
