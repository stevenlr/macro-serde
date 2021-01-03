use macroserde::{de, ser};
use std::io;
use std::io::{Read, Write};

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

pub struct Deserializer<R: io::Read> {
    read: io::BufReader<R>,
}

impl<R: io::Read> Deserializer<R> {
    pub fn new(read: R) -> Self {
        Self {
            read: io::BufReader::new(read),
        }
    }

    #[inline]
    fn read_u8(&mut self) -> Result<u8, de::DeserializeError> {
        let mut byte = [0];
        self.read.read_exact(&mut byte)?;
        return Ok(byte[0]);
    }

    #[inline]
    fn read_u16(&mut self) -> Result<u16, de::DeserializeError> {
        let mut byte = [0; 2];
        self.read.read_exact(&mut byte)?;
        Ok(u16::from_be_bytes(byte))
    }

    #[inline]
    fn read_u32(&mut self) -> Result<u32, de::DeserializeError> {
        let mut byte = [0; 4];
        self.read.read_exact(&mut byte)?;
        Ok(u32::from_be_bytes(byte))
    }

    #[inline]
    fn read_u64(&mut self) -> Result<u64, de::DeserializeError> {
        let mut byte = [0; 8];
        self.read.read_exact(&mut byte)?;
        Ok(u64::from_be_bytes(byte))
    }

    #[inline]
    fn read_i8(&mut self) -> Result<i8, de::DeserializeError> {
        let mut byte = [0];
        self.read.read_exact(&mut byte)?;
        return Ok(i8::from_be_bytes(byte));
    }

    #[inline]
    fn read_i16(&mut self) -> Result<i16, de::DeserializeError> {
        let mut byte = [0; 2];
        self.read.read_exact(&mut byte)?;
        Ok(i16::from_be_bytes(byte))
    }

    #[inline]
    fn read_i32(&mut self) -> Result<i32, de::DeserializeError> {
        let mut byte = [0; 4];
        self.read.read_exact(&mut byte)?;
        Ok(i32::from_be_bytes(byte))
    }

    #[inline]
    fn read_i64(&mut self) -> Result<i64, de::DeserializeError> {
        let mut byte = [0; 8];
        self.read.read_exact(&mut byte)?;
        Ok(i64::from_be_bytes(byte))
    }

    #[inline]
    fn read_f32(&mut self) -> Result<f32, de::DeserializeError> {
        let mut byte = [0; 4];
        self.read.read_exact(&mut byte)?;
        Ok(f32::from_be_bytes(byte))
    }

    #[inline]
    fn read_f64(&mut self) -> Result<f64, de::DeserializeError> {
        let mut byte = [0; 8];
        self.read.read_exact(&mut byte)?;
        Ok(f64::from_be_bytes(byte))
    }

    fn parse(&mut self, visitor: &mut dyn de::Visitor) -> Result<(), de::DeserializeError> {
        match self.read_u8()? {
            val @ 0x80..=0x8f => {
                self.parse_map((val - 0x80) as usize, &mut *visitor.visit_struct()?)
            }
            0xde => {
                let len = self.read_u16()? as usize;
                self.parse_map(len, &mut *visitor.visit_struct()?)
            }
            0xdf => {
                let len = self.read_u32()? as usize;
                self.parse_map(len, &mut *visitor.visit_struct()?)
            }
            val @ 0x90..=0x9f => {
                let len = (val - 0x90) as usize;
                self.parse_array(len, &mut *visitor.visit_seq(Some(len))?)
            }
            0xdc => {
                let len = self.read_u16()? as usize;
                self.parse_array(len, &mut *visitor.visit_seq(Some(len))?)
            }
            0xdd => {
                let len = self.read_u32()? as usize;
                self.parse_array(len, &mut *visitor.visit_seq(Some(len))?)
            }
            discriminant @ 0x00..=0x7f | discriminant @ 0xcc..=0xcf => {
                let value = self.parse_unsigned(discriminant)?;
                visitor.visit_unsigned(value)
            }
            discriminant @ 0xe0..=0xff | discriminant @ 0xd0..=0xd3 => {
                let value = self.parse_signed(discriminant)?;
                visitor.visit_signed(value)
            }
            0xca => {
                let value = self.read_f32()?;
                visitor.visit_float(value as f64)
            }
            0xcb => {
                let value = self.read_f64()?;
                visitor.visit_float(value)
            }
            0xc0 => visitor.visit_null(),
            0xc2 => visitor.visit_bool(false),
            0xc3 => visitor.visit_bool(true),
            val @ 0xa0..=0xbf => self.parse_str((val - 0xa0) as usize, visitor),
            0xd9 => {
                let len = self.read_u8()? as usize;
                self.parse_str(len, visitor)
            }
            0xda => {
                let len = self.read_u16()? as usize;
                self.parse_str(len, visitor)
            }
            0xdb => {
                let len = self.read_u32()? as usize;
                self.parse_str(len, visitor)
            }
            _ => Err(de::DeserializeError::ParsingError),
        }
    }

    fn parse_unsigned(&mut self, disciminant: u8) -> Result<u64, de::DeserializeError> {
        match disciminant {
            val @ 0x00..=0x7f => Ok(val as u64),
            0xcc => Ok(self.read_u8()? as u64),
            0xcd => Ok(self.read_u16()? as u64),
            0xce => Ok(self.read_u32()? as u64),
            0xcf => Ok(self.read_u64()? as u64),
            _ => Err(de::DeserializeError::ParsingError),
        }
    }

    fn parse_signed(&mut self, disciminant: u8) -> Result<i64, de::DeserializeError> {
        match disciminant {
            val @ 0xe0..=0xff => Ok(val as i8 as i64),
            0xd0 => Ok(self.read_i8()? as i64),
            0xd1 => Ok(self.read_i16()? as i64),
            0xd2 => Ok(self.read_i32()? as i64),
            0xd3 => Ok(self.read_i64()? as i64),
            _ => Err(de::DeserializeError::ParsingError),
        }
    }

    fn parse_map(
        &mut self,
        len: usize,
        builder: &mut dyn de::StructBuilder,
    ) -> Result<(), de::DeserializeError> {
        for _ in 0..len {
            let id_d = self.read_u8()?;
            let id = self.parse_unsigned(id_d)? as u32;
            self.parse(builder.member(Some(id), None)?)?;
        }
        builder.finish()
    }

    fn parse_array(
        &mut self,
        len: usize,
        builder: &mut dyn de::SeqBuilder,
    ) -> Result<(), de::DeserializeError> {
        for _ in 0..len {
            self.parse(builder.element()?)?;
        }
        builder.finish()
    }

    fn parse_str(
        &mut self,
        len: usize,
        visitor: &mut dyn de::Visitor,
    ) -> Result<(), de::DeserializeError> {
        let mut buffer = Vec::new();
        buffer.resize(len, 0);
        self.read.read_exact(&mut buffer)?;
        let s = std::str::from_utf8(&buffer).map_err(|_| de::DeserializeError::ParsingError)?;
        visitor.visit_str(&s)
    }
}

impl<R: io::Read> de::Deserializer for Deserializer<R> {
    fn deserialize(&mut self, visitor: &mut dyn de::Visitor) -> Result<(), de::DeserializeError> {
        self.parse(visitor)
    }
}
