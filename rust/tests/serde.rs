#![cfg(feature = "serde")]

use std::collections::{BTreeMap, HashMap};

use lwon::{Location, Parser, Value};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct Config {
    name: String,
    port: u16,
    ratio: f64,
    verbose: bool,
    initial: char,
    nickname: Option<String>,
    motto: Option<String>,
}

#[test]
fn scalars_are_parsed_from_text() {
    let c: Config = lwon::from_str(
        "{ name: server one\n port: 8080\n ratio: 0.5\n verbose: true\n initial: s\n motto: \"\" }",
    )
    .unwrap();
    assert_eq!(
        c,
        Config {
            name: "server one".into(),
            port: 8080,
            ratio: 0.5,
            verbose: true,
            initial: 's',
            nickname: None,
            motto: Some("".into()),
        }
    );
}

#[derive(Debug, Deserialize, PartialEq)]
struct Person {
    name: String,
    #[serde(rename = "friend", default)]
    friends: Vec<String>,
    #[serde(rename = "pet", default)]
    pets: Vec<Pet>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Pet {
    name: String,
    species: String,
}

#[test]
fn repeated_keys_form_a_sequence() {
    let p: Person = lwon::from_str(
        "{ friend: bob\n name: Alice\n friend: carol\n \
         pet { name: fido\n species: dog }\n pet { name: tom\n species: cat } }",
    )
    .unwrap();
    assert_eq!(p.name, "Alice");
    assert_eq!(p.friends, ["bob", "carol"]);
    let species: Vec<&str> = p.pets.iter().map(|p| p.species.as_str()).collect();
    assert_eq!(species, ["dog", "cat"]);
}

#[test]
fn a_single_value_is_a_sequence_of_one_unless_it_is_an_array() {
    let p: Person = lwon::from_str("{ name: Alice\n friend: bob }").unwrap();
    assert_eq!(p.friends, ["bob"]);
    let p: Person = lwon::from_str("{ name: Alice\n friend [ bob, carol ] }").unwrap();
    assert_eq!(p.friends, ["bob", "carol"]);
    let p: Person = lwon::from_str("{ name: Alice\n pet { name: tom\n species: cat } }").unwrap();
    assert_eq!(
        p.pets,
        [Pet {
            name: "tom".into(),
            species: "cat".into()
        }]
    );
}

#[test]
fn a_repeated_key_for_a_non_sequence_is_an_error() {
    let e = lwon::from_str::<Person>("{ name: Alice\n name: Bob }").unwrap_err();
    assert_eq!(e.message(), "key `name` is repeated");
    assert_eq!(e.location(), Some(Location { line: 2, column: 8 }));
}

#[test]
fn repeated_keys_in_maps() {
    let m: HashMap<String, Vec<String>> = lwon::from_str("{ a: 1\n b: 2\n a: 3 }").unwrap();
    assert_eq!(m["a"], ["1", "3"]);
    assert_eq!(m["b"], ["2"]);
    let m: BTreeMap<String, u32> = lwon::from_str("{ a: 1\n b: 2 }").unwrap();
    assert_eq!(
        m.into_iter().collect::<Vec<_>>(),
        [("a".into(), 1), ("b".into(), 2)]
    );
}

#[derive(Debug, Deserialize, PartialEq)]
struct Country {
    name: String,
    population: u32,
    gdp: f64,
}

#[test]
fn csv_rows_are_structs() {
    let mut parser = Parser::new("USA, 338, 23.3\nChina, 1411, 12.2\n", "countries.csv");
    let table = Value::from(parser.parse_implicit_array().unwrap().unwrap());
    let countries: Vec<Country> = lwon::from_value(&table).unwrap();
    assert_eq!(
        countries[1],
        Country {
            name: "China".into(),
            population: 1411,
            gdp: 12.2
        }
    );
}

#[test]
fn multidimensional_arrays_are_nested_sequences() {
    let v: Vec<Vec<u32>> = lwon::from_str("[1, 2, 3\n 4, 5, 6]").unwrap();
    assert_eq!(v, [[1, 2, 3], [4, 5, 6]]);
    let v: Vec<Vec<Vec<String>>> = lwon::from_str("[a, b\n c, d\n\n e, f\n g, h]").unwrap();
    assert_eq!(v[1][0], ["e", "f"]);
    let v: Vec<(String, u32)> = lwon::from_str("[a, 1\n b, 2]").unwrap();
    assert_eq!(v, [("a".into(), 1), ("b".into(), 2)]);
}

#[test]
fn extra_elements_are_an_error() {
    let e = lwon::from_str::<(u32, u32)>("[1, 2, 3]").unwrap_err();
    assert_eq!(e.message(), "invalid length 3, expected 2 elements");
    assert_eq!(e.location(), Some(Location { line: 1, column: 1 }));
}

#[derive(Debug, Deserialize, PartialEq)]
enum Shape {
    Point,
    Circle(f64),
    Rect { width: u32, height: u32 },
}

#[test]
fn enums() {
    let shapes: Vec<Shape> =
        lwon::from_str("[ Point, { Circle: 1.5 }, { Rect { width: 2\n height: 3 } } ]").unwrap();
    assert_eq!(
        shapes,
        [
            Shape::Point,
            Shape::Circle(1.5),
            Shape::Rect {
                width: 2,
                height: 3
            }
        ]
    );
    let e = lwon::from_str::<Shape>("Square").unwrap_err();
    assert!(e.message().starts_with("unknown variant `Square`"), "{e}");
}

#[test]
fn errors_give_the_location_of_the_value() {
    let e = lwon::from_str::<Config>("{\n name: x\n port: eighty }").unwrap_err();
    assert_eq!(e.location(), Some(Location { line: 3, column: 8 }));
    assert_eq!(
        e.to_string(),
        "\"<string>\", line 3, char 8: invalid value: string \"eighty\", expected u16"
    );
    let value: Value = "\n  { name: x }".parse().unwrap();
    let e = lwon::from_value::<Config>(&value).unwrap_err();
    assert_eq!(e.to_string(), "line 2, char 3: missing field `port`");
}

#[test]
fn from_value_borrows_strings() {
    #[derive(Deserialize)]
    struct Borrowed<'a> {
        name: &'a str,
    }
    let value: Value = "{ name: Alice }".parse().unwrap();
    let b: Borrowed = lwon::from_value(&value).unwrap();
    assert_eq!(b.name, "Alice");
}

#[test]
fn empty_text_is_unit() {
    let units: Vec<()> = lwon::from_str("[\"\", \"\"]").unwrap();
    assert_eq!(units.len(), 2);
    assert!(lwon::from_str::<()>("x").is_err());
}

#[test]
fn repeated_keys_are_not_tuples_or_fixed_size_arrays() {
    #[derive(Debug, Deserialize)]
    struct Pair {
        #[allow(dead_code)]
        p: [u32; 2],
    }
    let e = lwon::from_str::<Pair>("{ p: 1\n p: 2 }").unwrap_err();
    assert_eq!(e.message(), "key `p` is repeated");
    let ok: Pair = lwon::from_str("{ p [1, 2] }").unwrap();
    assert_eq!(ok.p, [1, 2]);
    let set: std::collections::BTreeSet<u32> =
        lwon::from_str::<HashMap<String, std::collections::BTreeSet<u32>>>("{ s: 2\n s: 1 }")
            .unwrap()
            .remove("s")
            .unwrap();
    assert_eq!(set.into_iter().collect::<Vec<_>>(), [1, 2]);
}
