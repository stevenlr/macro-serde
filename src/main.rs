mod const_assert;
mod de;
mod macros;
mod ser;

use de::*;
use ser::*;
use std::fmt::Write;
use std::iter::{Enumerate, Peekable};
use std::ops::Range;
use std::str::Chars;

struct JsonDeserializer<'a> {
    data: &'a str,
    iter: Peekable<Enumerate<Chars<'a>>>,
}

impl<'a> JsonDeserializer<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            data: s,
            iter: s.chars().enumerate().peekable(),
        }
    }

    #[inline]
    fn peek_char(&mut self) -> Option<char> {
        self.iter.peek().map(|(_, c)| *c)
    }

    #[inline]
    fn next_char(&mut self) -> Option<char> {
        self.iter.next().map(|(_, c)| c)
    }

    fn parse_integer(&mut self, visitor: &mut dyn Visitor) -> Result<(), DeserializeError> {
        let is_negative = if matches!(self.peek_char(), Some('-')) {
            self.iter.next();
            true
        } else {
            false
        };

        // @Todo Proper overflow check when parsing

        let mut int_part: u64 = 0;

        loop {
            match self.peek_char() {
                Some(c @ '0'..='9') => {
                    let c = c.to_digit(10).unwrap() as u64;
                    if int_part > (std::u64::MAX - c) / 10 {
                        return Err(DeserializeError::IncompatibleNumericType);
                    }
                    int_part = int_part * 10 + c;
                    self.iter.next();
                }
                _ => break,
            }
        }

        let mut fract_part: f64 = 0.0;
        let mut fract_part_mul: f64 = 0.1;
        if matches!(self.peek_char(), Some('.')) {
            self.iter.next();
            loop {
                match self.peek_char() {
                    Some(c @ '0'..='9') => {
                        let c = c.to_digit(10).unwrap() as f64;
                        fract_part = fract_part + fract_part_mul * c;
                        fract_part_mul *= 0.1;
                        self.iter.next();
                    }
                    _ => break,
                }
            }
        }

        if fract_part != 0.0 {
            let float_value = if is_negative { -1.0 } else { 1.0 } * (int_part as f64 + fract_part);
            return visitor.visit_float(float_value);
        } else if is_negative {
            if int_part <= (std::i64::MIN as u64).wrapping_neg() {
                let value = (int_part as i64).wrapping_neg();
                return visitor.visit_signed(value);
            } else {
                return Err(DeserializeError::IncompatibleNumericType);
            }
        } else {
            if int_part < std::i64::MAX as u64 {
                return visitor.visit_signed(int_part as i64);
            } else {
                return visitor.visit_unsigned(int_part);
            }
        }
    }

    fn check_keywork(&mut self, kw: &'static str) -> bool {
        if kw
            .chars()
            .all(|c| matches!(self.next_char(), Some(r) if r == c))
        {
            !self.peek_char().filter(|c| c.is_alphanumeric()).is_some()
        } else {
            false
        }
    }

    fn parse_null(&mut self, visitor: &mut dyn Visitor) -> Result<(), DeserializeError> {
        if self.check_keywork("null") {
            return visitor.visit_null();
        } else {
            Err(DeserializeError::ParsingError)
        }
    }

    fn parse_true(&mut self, visitor: &mut dyn Visitor) -> Result<(), DeserializeError> {
        if self.check_keywork("true") {
            return visitor.visit_bool(true);
        } else {
            Err(DeserializeError::ParsingError)
        }
    }

    fn parse_false(&mut self, visitor: &mut dyn Visitor) -> Result<(), DeserializeError> {
        if self.check_keywork("false") {
            return visitor.visit_bool(false);
        } else {
            Err(DeserializeError::ParsingError)
        }
    }

    fn parse_str_ref(&mut self) -> Result<Range<usize>, DeserializeError> {
        let first_index = match self.iter.next() {
            Some((i, '"')) => i + 1,
            _ => return Err(DeserializeError::ParsingError),
        };

        // @Todo Handle escaped characters

        let last_index = loop {
            match self.iter.next() {
                Some((i, '"')) => break i,
                None => return Err(DeserializeError::UnexpectedEof),
                _ => {}
            }
        };

        return Ok(first_index..last_index);
    }

    fn parse_str(&mut self, visitor: &mut dyn Visitor) -> Result<(), DeserializeError> {
        visitor.visit_str(&self.data[self.parse_str_ref()?])?;
        Ok(())
    }

    fn parse_sequence(&mut self, visitor: &mut dyn Visitor) -> Result<(), DeserializeError> {
        self.iter.next();

        let mut seq = visitor.visit_seq(None)?;

        loop {
            if self.peek_char() == Some(']') {
                self.iter.next();
                return seq.finish();
            }

            self.deserialize(seq.element()?)?;

            match self.peek_char() {
                Some(',') => {
                    self.next_char();
                }
                Some(']') => {}
                _ => return Err(DeserializeError::ParsingError),
            }
        }
    }

    fn parse_field_name(&mut self) -> Result<(Option<u64>, Range<usize>), DeserializeError> {
        let range = self.parse_str_ref()?;
        let s = &self.data[range.clone()];
        if let Some(index) = s.find(':') {
            let id = self.data[range.start..(range.start + index)]
                .parse::<u64>()
                .ok();
            return Ok((id, (range.start + index + 1)..range.end));
        } else {
            return Ok((None, range));
        }
    }

    fn parse_struct(&mut self, visitor: &mut dyn Visitor) -> Result<(), DeserializeError> {
        self.iter.next();

        let mut struc = visitor.visit_struct()?;

        loop {
            if self.peek_char() == Some('}') {
                self.iter.next();
                return struc.finish();
            }

            let (id, name_range) = self.parse_field_name()?;

            if self.next_char() != Some(':') {
                return Err(DeserializeError::ParsingError);
            }

            self.deserialize(struc.member(id, Some(&self.data[name_range]))?)?;

            match self.peek_char() {
                Some(',') => {
                    self.next_char();
                }
                Some('}') => {}
                _ => return Err(DeserializeError::ParsingError),
            }
        }
    }
}

impl<'a> Deserializer for JsonDeserializer<'a> {
    fn deserialize(&mut self, visitor: &mut dyn Visitor) -> Result<(), DeserializeError> {
        match self.peek_char() {
            Some('0'..='9') | Some('-') => self.parse_integer(visitor),
            Some('n') => self.parse_null(visitor),
            Some('t') => self.parse_true(visitor),
            Some('f') => self.parse_false(visitor),
            Some('"') => self.parse_str(visitor),
            Some('[') => self.parse_sequence(visitor),
            Some('{') => self.parse_struct(visitor),
            Some(_) => Err(DeserializeError::ParsingError),
            None => Err(DeserializeError::UnexpectedEof),
        }
    }
}

#[derive(Default)]
struct PrettyJsonSerializer {
    indent_level: usize,
    buffer: String,
}

impl PrettyJsonSerializer {
    fn print_indent(&mut self) -> Result<(), SerializeError> {
        for _ in 0..self.indent_level {
            write!(&mut self.buffer, "    ")?;
        }
        Ok(())
    }
}

impl Serializer for PrettyJsonSerializer {
    fn serialize_null(&mut self) -> Result<(), SerializeError> {
        write!(&mut self.buffer, "null")?;
        Ok(())
    }

    fn serialize_bool(&mut self, value: bool) -> Result<(), SerializeError> {
        if value {
            write!(&mut self.buffer, "true")?;
        } else {
            write!(&mut self.buffer, "false")?;
        }
        Ok(())
    }

    fn serialize_signed(&mut self, value: i64) -> Result<(), SerializeError> {
        write!(&mut self.buffer, "{}", value)?;
        Ok(())
    }

    fn serialize_unsigned(&mut self, value: u64) -> Result<(), SerializeError> {
        write!(&mut self.buffer, "{}", value)?;
        Ok(())
    }

    fn serialize_float(&mut self, value: f64) -> Result<(), SerializeError> {
        write!(&mut self.buffer, "{}", value)?;
        Ok(())
    }

    fn serialize_str(&mut self, value: &str) -> Result<(), SerializeError> {
        write!(&mut self.buffer, "\"{}\"", value)?;
        Ok(())
    }

    fn serialize_enum(&mut self, value: u32, name: &'static str) -> Result<(), SerializeError> {
        write!(&mut self.buffer, "\"{}:{}\"", value, name)?;
        Ok(())
    }

    fn start_struct(&mut self) -> Result<(), SerializeError> {
        writeln!(&mut self.buffer, "{{")?;
        self.indent_level += 1;
        Ok(())
    }

    fn serialize_struct_field(
        &mut self,
        field_id: u32,
        field_name: &'static str,
        value: &dyn Serialize,
    ) -> Result<(), SerializeError> {
        self.print_indent()?;
        write!(&mut self.buffer, "\"{}:{}\": ", field_id, field_name)?;
        value.serialize(self)?;
        writeln!(&mut self.buffer, ",")?;
        Ok(())
    }

    fn end_struct(&mut self) -> Result<(), SerializeError> {
        self.indent_level -= 1;
        self.print_indent()?;
        write!(&mut self.buffer, "}}")?;
        Ok(())
    }

    fn start_seq(&mut self, _len: usize) -> Result<(), SerializeError> {
        writeln!(&mut self.buffer, "[")?;
        self.indent_level += 1;
        Ok(())
    }

    fn serialize_seq_elmt(&mut self, value: &dyn Serialize) -> Result<(), SerializeError> {
        self.print_indent()?;
        value.serialize(self)?;
        writeln!(&mut self.buffer, ",")?;
        Ok(())
    }

    fn end_seq(&mut self) -> Result<(), SerializeError> {
        self.indent_level -= 1;
        self.print_indent()?;
        write!(&mut self.buffer, "]")?;
        Ok(())
    }
}

#[derive(Default)]
struct JsonSerializer {
    buffer: String,
}

impl Serializer for JsonSerializer {
    fn serialize_null(&mut self) -> Result<(), SerializeError> {
        write!(&mut self.buffer, "null")?;
        Ok(())
    }

    fn serialize_bool(&mut self, value: bool) -> Result<(), SerializeError> {
        if value {
            write!(&mut self.buffer, "true")?;
        } else {
            write!(&mut self.buffer, "false")?;
        }
        Ok(())
    }

    fn serialize_signed(&mut self, value: i64) -> Result<(), SerializeError> {
        write!(&mut self.buffer, "{}", value)?;
        Ok(())
    }

    fn serialize_unsigned(&mut self, value: u64) -> Result<(), SerializeError> {
        write!(&mut self.buffer, "{}", value)?;
        Ok(())
    }

    fn serialize_float(&mut self, value: f64) -> Result<(), SerializeError> {
        write!(&mut self.buffer, "{}", value)?;
        Ok(())
    }

    fn serialize_str(&mut self, value: &str) -> Result<(), SerializeError> {
        write!(&mut self.buffer, "\"{}\"", value)?;
        Ok(())
    }

    fn serialize_enum(&mut self, value: u32, name: &'static str) -> Result<(), SerializeError> {
        write!(&mut self.buffer, "\"{}:{}\"", value, name)?;
        Ok(())
    }

    fn start_struct(&mut self) -> Result<(), SerializeError> {
        write!(&mut self.buffer, "{{")?;
        Ok(())
    }

    fn serialize_struct_field(
        &mut self,
        field_id: u32,
        field_name: &'static str,
        value: &dyn Serialize,
    ) -> Result<(), SerializeError> {
        write!(&mut self.buffer, "\"{}:{}\":", field_id, field_name)?;
        value.serialize(self)?;
        write!(&mut self.buffer, ",")?;
        Ok(())
    }

    fn end_struct(&mut self) -> Result<(), SerializeError> {
        write!(&mut self.buffer, "}}")?;
        Ok(())
    }

    fn start_seq(&mut self, _len: usize) -> Result<(), SerializeError> {
        write!(&mut self.buffer, "[")?;
        Ok(())
    }

    fn serialize_seq_elmt(&mut self, value: &dyn Serialize) -> Result<(), SerializeError> {
        value.serialize(self)?;
        write!(&mut self.buffer, ",")?;
        Ok(())
    }

    fn end_seq(&mut self) -> Result<(), SerializeError> {
        write!(&mut self.buffer, "]")?;
        Ok(())
    }
}

serde! {
    #[derive(Debug, PartialEq)]
    enum Month {
        January = 1,
        February = 2,
        March = 3,
        April = 4,
        May = 5,
        June = 6,
        July = 7,
        August = 8,
        September = 9,
        October = 10,
        November = 11,
        December = 12,
    }
}

serde! {
    #[derive(Debug, PartialEq)]
    struct Date {
        day: u32 = 1,
        month: Month = 2,
        year: u32 = 3,
    }
}

serde! {
    #[derive(Debug, PartialEq)]
    struct Person {
        name: String = 1,
        age: i32 = 2,
        birth_date: Date = 3,
        pets: Vec<String> = 4,
        height: Option<f32> = 5,
        weight: Option<f32> = 88,
        is_cool: bool = 6,
    }
}

#[test]
fn de() {
    let mut de = JsonDeserializer::new("123");
    assert_eq!(i32::deserialize(&mut de), Ok(123));

    let mut de = JsonDeserializer::new("-123");
    assert_eq!(i32::deserialize(&mut de), Ok(-123));

    let mut de = JsonDeserializer::new("0");
    assert_eq!(i32::deserialize(&mut de), Ok(0));

    let mut de = JsonDeserializer::new("3000000000");
    assert_eq!(
        i32::deserialize(&mut de),
        Err(DeserializeError::IncompatibleNumericType)
    );

    let mut de = JsonDeserializer::new("-456.21");
    assert_eq!(i32::deserialize(&mut de), Ok(-456));

    let mut de = JsonDeserializer::new("123");
    assert_eq!(u32::deserialize(&mut de), Ok(123));

    let mut de = JsonDeserializer::new("-123");
    assert!(u32::deserialize(&mut de).is_err());

    let mut de = JsonDeserializer::new("0");
    assert_eq!(u32::deserialize(&mut de), Ok(0));

    let mut de = JsonDeserializer::new("3000000000");
    assert_eq!(u32::deserialize(&mut de), Ok(3000000000));

    let mut de = JsonDeserializer::new("456.21");
    assert_eq!(u32::deserialize(&mut de), Ok(456));

    let mut de = JsonDeserializer::new("456.21");
    assert_eq!(f32::deserialize(&mut de), Ok(456.21));

    let mut de = JsonDeserializer::new("-456.21");
    assert_eq!(f32::deserialize(&mut de), Ok(-456.21));

    let mut de = JsonDeserializer::new("0.21");
    assert_eq!(f32::deserialize(&mut de), Ok(0.21));

    let mut de = JsonDeserializer::new("-0.21");
    assert_eq!(f32::deserialize(&mut de), Ok(-0.21));

    let mut de = JsonDeserializer::new("123");
    assert_eq!(f32::deserialize(&mut de), Ok(123.0));

    let mut de = JsonDeserializer::new("-123");
    assert_eq!(f32::deserialize(&mut de), Ok(-123.0));

    let mut de = JsonDeserializer::new("87");
    assert_eq!(Option::<i32>::deserialize(&mut de), Ok(Some(87)));

    let mut de = JsonDeserializer::new("null");
    assert_eq!(Option::<i32>::deserialize(&mut de), Ok(None));

    let mut de = JsonDeserializer::new("84");
    assert_eq!(bool::deserialize(&mut de), Ok(true));

    let mut de = JsonDeserializer::new("0");
    assert_eq!(bool::deserialize(&mut de), Ok(false));

    let mut de = JsonDeserializer::new("true");
    assert_eq!(bool::deserialize(&mut de), Ok(true));

    let mut de = JsonDeserializer::new("false");
    assert_eq!(bool::deserialize(&mut de), Ok(false));

    let mut de = JsonDeserializer::new("\"hello world :)\"");
    assert!(matches!(String::deserialize(&mut de), Ok(ref s) if s == "hello world :)"));

    let mut de = JsonDeserializer::new("\"\"");
    assert!(matches!(String::deserialize(&mut de), Ok(ref s) if s == ""));

    let mut de = JsonDeserializer::new("\"10:October\"");
    assert!(matches!(Month::deserialize(&mut de), Ok(Month::October)));

    let mut de = JsonDeserializer::new("\"October\"");
    assert!(matches!(Month::deserialize(&mut de), Ok(Month::October)));

    let mut de = JsonDeserializer::new("10");
    assert!(matches!(Month::deserialize(&mut de), Ok(Month::October)));

    let mut de = JsonDeserializer::new("[1,2,4,8]");
    assert!(matches!(Vec::<i32>::deserialize(&mut de), Ok(ref s) if s == &[1, 2, 4, 8]));

    let mut de = JsonDeserializer::new("[1,2,4,8,]");
    assert!(matches!(Vec::<i32>::deserialize(&mut de), Ok(ref s) if s == &[1, 2, 4, 8]));

    let mut de = JsonDeserializer::new("[1]");
    assert!(matches!(Vec::<i32>::deserialize(&mut de), Ok(ref s) if s == &[1]));

    let mut de = JsonDeserializer::new("[1,null,2,null]");
    assert!(
        matches!(Vec::<Option<i32>>::deserialize(&mut de), Ok(ref s) if s == &[Some(1), None, Some(2), None])
    );

    let mut de = JsonDeserializer::new("[]");
    assert!(matches!(Vec::<i32>::deserialize(&mut de), Ok(ref s) if s == &[]));
}

fn main() {
    let stuff = Person {
        name: "Steven".to_owned(),
        age: 27,
        height: Some(1.735),
        weight: None,
        is_cool: true,
        birth_date: Date {
            day: 19,
            month: Month::October,
            year: 1993,
        },
        pets: vec!["Bouboul".to_owned(), "Monsieur Puppy".to_owned()],
    };

    let mut serializer = JsonSerializer::default();
    stuff.serialize(&mut serializer).unwrap();
    println!("{}", serializer.buffer);

    let mut de = JsonDeserializer::new(&serializer.buffer);
    assert_eq!(Person::deserialize(&mut de), Ok(stuff));
}
