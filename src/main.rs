mod const_assert;
mod macros;
mod ser;

use ser::*;
use std::fmt::Write;

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
    fn serialize_signed(&mut self, value: i64) -> Result<(), SerializeError> {
        write!(&mut self.buffer, "{}", value)?;
        Ok(())
    }

    fn serialize_unsigned(&mut self, value: u64) -> Result<(), SerializeError> {
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

serde! {
    #[derive(Debug)]
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
    #[derive(Debug)]
    struct Date {
        day: u32 = 1,
        month: Month = 2,
        year: u32 = 3,
    }
}

serde! {
    #[derive(Debug)]
    struct Person {
        name: String = 1,
        age: i32 = 2,
        birth_date: Date = 3,
        pets: Vec<String> = 4,
    }
}

fn main() {
    let stuff = Person {
        name: "Steven".to_owned(),
        age: 27,
        birth_date: Date {
            day: 19,
            month: Month::October,
            year: 1993,
        },
        pets: vec!["Bouboul".to_owned(), "Monsieur Puppy".to_owned()],
    };

    let mut serializer = PrettyJsonSerializer::default();
    stuff.serialize(&mut serializer).unwrap();
    println!("{}", serializer.buffer);
}
