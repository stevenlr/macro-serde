This is an experimental serialization framework for Rust that uses macros instead of procedural macros to generate the serialization and deserialization code. The in-memory representation of the deserialized data is exactly what one could expect from defining the same structures in Rust.

Like [miniserde](https://github.com/dtolnay/miniserde) it avoids monomorphization, and tries to be minimal. However it supports enumerations and discriminated unions (Rust's enums basically).

Like [protobuf](https://github.com/protocolbuffers/protobuf), struct fields, enum variants, and union variants, are identified by unique identifiers. Those identifiers are checked for unicity at compile-time. Serializers can use the ID, the name, or both to identify them. This is useful for making the serialization formats human-readable. Deserializers can check the ID when it's available or the name when it's available. The ID is supposed to be more reliable but in human-edited data, only the field/variant name may be present.

Example
-------------

```rust
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
        day: u8 = 1,
        month: Month = 2,
        year: u32 = 3,
    }
}

serde! {
    #[derive(Debug, PartialEq)]
    union Occupation {
        Unemployed = 1,
        Employed(String) = 2,
    }
}

serde! {
    #[derive(Debug, PartialEq)]
    struct Person {
        name: String = 1,
        age: i16 = 2,
        birth_date: Date = 3,
        pets: Vec<String> = 4,
        height: Option<f32> = 5,
        car_brand: Option<String> = 88,
        is_cool: bool = 6,
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

    let mut serializer = PrettyJsonSerializer::default();
    stuff.serialize(&mut serializer).unwrap();
    println!("{}", serializer.buffer);
}
```

The code above would produce:


```json
{                                  
    "1:name": "Steven",            
    "2:age": 27,                   
    "3:birth_date": {              
        "1:day": 19,               
        "2:month": "10:October",   
        "3:year": 1993,            
    },                             
    "4:pets": [                    
        "Bouboul",                 
        "Monsieur Puppy",          
    ],                             
    "5:height": 1.7350000143051147,
    "88:car_brand": null,          
    "6:is_cool": true,             
    "7:occupation": {              
        "2:Employed": "Engineer",  
    },                             
}                                  
```

Future work
-----------------

- Reserved IDs.
- Default values.
- Proper JSON implementation.
- MessagePack implementation.
- Improve compile-time ID unicity check error message. https://github.com/rust-lang/rust/issues/51999