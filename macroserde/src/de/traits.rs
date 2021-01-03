use std::io;

#[derive(Debug)]
pub enum DeserializeError {
    UnknownError,
    UnimplementedVisit,
    IncompatibleNumericType,
    UnexpectedEof,
    UnknownEnumVariant,
    UnknownUnionVariant,
    ParsingError,
    MissingField(&'static str),
    UnknownField,
    IoError(io::Error),
}

pub trait SeqBuilder {
    fn element(&mut self) -> Result<&mut dyn Visitor, DeserializeError>;
    fn finish(&mut self) -> Result<(), DeserializeError>;
}

pub trait StructBuilder {
    fn member(
        &mut self,
        id: Option<u32>,
        name: Option<&str>,
    ) -> Result<&mut dyn Visitor, DeserializeError>;
    fn finish(&mut self) -> Result<(), DeserializeError>;
}

pub trait Visitor {
    fn visit_null(&mut self) -> Result<(), DeserializeError> {
        Err(DeserializeError::UnimplementedVisit)
    }

    fn visit_bool(&mut self, _value: bool) -> Result<(), DeserializeError> {
        Err(DeserializeError::UnimplementedVisit)
    }

    fn visit_signed(&mut self, _value: i64) -> Result<(), DeserializeError> {
        Err(DeserializeError::UnimplementedVisit)
    }

    fn visit_unsigned(&mut self, _value: u64) -> Result<(), DeserializeError> {
        Err(DeserializeError::UnimplementedVisit)
    }

    fn visit_float(&mut self, _value: f64) -> Result<(), DeserializeError> {
        Err(DeserializeError::UnimplementedVisit)
    }

    fn visit_str(&mut self, _value: &str) -> Result<(), DeserializeError> {
        Err(DeserializeError::UnimplementedVisit)
    }

    fn visit_seq<'a>(
        &'a mut self,
        _size_hint: Option<usize>,
    ) -> Result<Box<dyn SeqBuilder + 'a>, DeserializeError> {
        Err(DeserializeError::UnimplementedVisit)
    }

    fn visit_struct<'a>(&'a mut self) -> Result<Box<dyn StructBuilder + 'a>, DeserializeError> {
        Err(DeserializeError::UnimplementedVisit)
    }
}

pub trait Deserializer {
    fn deserialize(&mut self, visitor: &mut dyn Visitor) -> Result<(), DeserializeError>;
}

pub trait Deserialize: Sized {
    fn deserialize(de: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
        let mut result = None;
        de.deserialize(Self::begin_deserialize(&mut result))?;
        return result.ok_or(DeserializeError::UnknownError);
    }

    fn begin_deserialize(out: &mut Option<Self>) -> &mut dyn Visitor;
}
