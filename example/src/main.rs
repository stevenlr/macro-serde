use macroserde::de::*;
use macroserde::macroserde;
use macroserde::ser::*;

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
    println!("");

    let mut result = Vec::<u8>::new();
    ser.write_pretty(&mut result, 2).unwrap();
    let result_str = std::str::from_utf8(&result).unwrap();

    let mut de = macroserde_json::Deserializer::new(&result_str).unwrap();
    let person = Person::deserialize(&mut de).unwrap();

    println!("{:#?}", person);

    assert_eq!(person, stuff);
}
