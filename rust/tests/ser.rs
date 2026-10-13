#![cfg(feature = "serde")]

use std::collections::BTreeMap;
use std::fmt::Debug;

use lwon::Value;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Serialize value, check that deserializing the text gives it back, and
/// return the text.
fn round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) -> String {
    let text = lwon::to_string(value).unwrap();
    assert_eq!(&lwon::from_str::<T>(&text).unwrap(), value, "{text}");
    text
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
    name: String,
    port: u16,
    ratio: f64,
    verbose: bool,
    initial: char,
    nickname: Option<String>,
    motto: Option<String>,
    tags: Vec<String>,
    limits: BTreeMap<String, u32>,
    shape: Shape,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Shape {
    Point,
    Circle(f64),
    Polygon(u32, u32),
    Rect { width: u32, height: u32 },
}

#[test]
fn structs_round_trip() {
    let config = Config {
        name: "server \"one\"".into(),
        port: 8080,
        ratio: 0.5,
        verbose: true,
        initial: 's',
        nickname: None,
        motto: Some("".into()),
        tags: vec!["a".into()],
        limits: [("cpu".into(), 4), ("memory size".into(), 512)]
            .into_iter()
            .collect(),
        shape: Shape::Rect {
            width: 2,
            height: 3,
        },
    };
    let text = round_trip(&config);
    assert!(!text.contains("nickname"), "{text}");
    assert!(text.contains("memory size: \"512\""), "{text}");
}

#[test]
fn enums_round_trip() {
    let shapes = vec![
        Shape::Point,
        Shape::Circle(1.5),
        Shape::Polygon(5, 1),
        Shape::Rect {
            width: 2,
            height: 3,
        },
    ];
    round_trip(&shapes);
    assert_eq!(lwon::to_string(&Shape::Point).unwrap(), "\"Point\"");
    assert_eq!(
        lwon::to_string(&Shape::Circle(1.5)).unwrap(),
        "{\n  Circle: \"1.5\"\n}"
    );
}

#[test]
fn nested_sequences_of_the_same_shape_are_multidimensional() {
    let value = lwon::to_value(&vec![vec![1, 2, 3], vec![4, 5, 6]]).unwrap();
    assert_eq!(value.as_array().unwrap().dimensions(), [2, 3]);
    let value = lwon::to_value(&vec![vec![vec!["a"]; 2]; 3]).unwrap();
    assert_eq!(value.as_array().unwrap().dimensions(), [3, 2, 1]);
    round_trip(&vec![vec![vec!["a".to_string(), "b".into()]; 2]; 3]);
    round_trip(&vec![vec![1], vec![2]]);
}

#[test]
fn other_nested_sequences_are_arrays_of_arrays() {
    for v in [
        vec![vec![1, 2, 3]],
        vec![vec![1, 2], vec![3]],
        vec![vec![], vec![]],
    ] {
        let value = lwon::to_value(&v).unwrap();
        assert_eq!(value.as_array().unwrap().dimensions(), [v.len()]);
        round_trip(&v);
    }
    round_trip(&Vec::<Vec<u32>>::new());
}

#[test]
fn none_outside_a_struct_or_map_is_an_error() {
    assert!(lwon::to_string(&None::<u32>).is_err());
    assert!(lwon::to_string(&vec![Some(1), None]).is_err());
    let m: BTreeMap<&str, Option<u32>> = [("a", Some(1)), ("b", None)].into_iter().collect();
    assert_eq!(lwon::to_string(&m).unwrap(), "{\n  a: \"1\"\n}");
}

#[test]
fn map_keys_are_text() {
    let m: BTreeMap<u32, bool> = [(1, true), (2, false)].into_iter().collect();
    round_trip(&m);
    let m: BTreeMap<(u32, u32), u32> = [((1, 2), 3)].into_iter().collect();
    let e = lwon::to_string(&m).unwrap_err();
    assert!(e.message().starts_with("dictionary keys must be"), "{e}");
}

#[test]
fn bytes_must_be_utf8() {
    #[derive(Serialize)]
    struct Bytes<'a>(#[serde(with = "serde_bytes_like")] &'a [u8]);
    mod serde_bytes_like {
        pub fn serialize<S: serde::Serializer>(v: &&[u8], s: S) -> Result<S::Ok, S::Error> {
            s.serialize_bytes(v)
        }
    }
    assert_eq!(lwon::to_string(&Bytes(b"abc")).unwrap(), "\"abc\"");
    assert!(lwon::to_string(&Bytes(&[0xff])).is_err());
}

#[test]
fn unit_is_empty_text() {
    round_trip(&());
    assert_eq!(lwon::to_value(&()).unwrap(), Value::from(""));
}

#[test]
fn special_characters_round_trip() {
    round_trip(&"line 1\nline 2 \u{1F600} \"quoted\" \\ / # { } [ ] :".to_string());
    round_trip(&vec![
        " leading".to_string(),
        "trailing ".into(),
        "a, b".into(),
        "".into(),
    ]);
    let m: BTreeMap<String, u32> = [
        ("".into(), 1),
        (" x".into(), 2),
        ("a:b".into(), 3),
        ("#c".into(), 4),
    ]
    .into_iter()
    .collect();
    round_trip(&m);
}
