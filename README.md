This is an experimental serialization framework for Rust that uses macros instead of procedural macros to generate the serialization and deserialization code. The in-memory representation of the deserialized data is exactly what one could expect from defining the same structures in Rust.

Like [miniserde](https://github.com/dtolnay/miniserde) it avoids monomorphization, and supports renaming. However it also supports enumerations and discriminated unions (Rust's enums basically).

Like [protobuf](https://github.com/protocolbuffers/protobuf), struct fields, enum variants, and union variants, are identified by unique identifiers. Those identifiers are checked for unicity at compile-time. Serializers can use the ID, the name, or both to identify them. This is useful for making the serialization formats human-readable. Deserializers can check the ID when it's available or the name when it's available. The ID is supposed to be more reliable but in human-edited data, only the field/variant name may be present.

Example
-------------

```rust
macroserde! {
    #[derive(Debug, PartialEq)]
    enum Month {
        January = 1 @ "JAN",
        February = 2 @ "FEB",
        March = 3 @ "MAR",
        April = 4 @ "APR",
        May = 5 @ "MAY",
        June = 6 @ "JUN",
        July = 7 @ "JUL",
        August = 8 @ "AUG",
        September = 9 @ "SEP",
        October = 10 @ "OCT",
        November = 11 @ "NOV",
        December = 12 @ "DEC",
    }
}

impl Default for Month {
    fn default() -> Self {
        Self::January
    }
}

macroserde! {
    #[derive(Debug, PartialEq, Default)]
    struct Date {
        day: u8 = 1,
        month: Month = 2,
        year: u32 = 3,
    }
}

macroserde! {
    #[derive(Debug, PartialEq)]
    union Occupation {
        Unemployed = 1,
        Employed(String) = 2 @ "hasJob",
    }
}

impl Default for Occupation {
    fn default() -> Self {
        Self::Unemployed
    }
}

macroserde! {
    #[derive(Debug, PartialEq, Default)]
    struct Person {
        name: String = 1,
        age: i16 = 2,
        birth_date: Date = 3,
        pets: Vec<String> = 4,
        height: Option<f32> = 5,
        car_brand: Option<String> = 88 @ "carBrand",
        is_cool: bool = 6 @ "IsCool",
        occupation: Occupation = 7,
    }
}

fn main() {
    let stuff = Person {
        name: "Steven".to_owned(),
        age: 27,
        height: Some(1.735),
        car_brand: None,
        is_cool: true,
        birth_date: Date {
            day: 19,
            month: Month::October,
            year: 1993,
        },
        pets: vec!["Bouboul".to_owned(), "Monsieur Puppy".to_owned()],
        occupation: Occupation::Employed("Engineer".to_owned()),
    };

    let mut ser = macroserde_json::Serializer::new();
    stuff.serialize(&mut ser).unwrap();

    ser.write_pretty(&mut std::io::stdout(), 2).unwrap();
}
```

The code above would produce:


```json
{
  "1:name": "Steven",
  "2:age": 27,
  "3:birth_date": {
    "1:day": 19,
    "2:month": "10:OCT",
    "3:year": 1993
  },
  "4:pets": [
    "Bouboul",
    "Monsieur Puppy"
  ],
  "5:height": 1.7350000143051148,
  "88:carBrand": null,
  "6:IsCool": true,
  "7:occupation": {
    "2:hasJob": "Engineer"
  }
}
```

Future work
-----------------

- Reserved IDs.
- Bytes value.
- MessagePack implementation.
- Improve compile-time ID unicity check error message. https://github.com/rust-lang/rust/issues/51999