use lwon::{Array, Dictionary, Location, Parser, Text, Value};

fn text(s: &str) -> Value {
    Value::from(s)
}

#[test]
fn repeated_keys_keep_all_values_in_order() {
    let value: Value = "{ a: 1\n b: 2\n a: 3 }".parse().unwrap();
    let d = value.as_dictionary().unwrap();
    let keys: Vec<&str> = d.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(keys, ["a", "b", "a"]);
    assert_eq!(d.get_all("a").collect::<Vec<_>>(), [&text("1"), &text("3")]);
    assert_eq!(d.get_first("a"), Some(&text("1")));
    assert_eq!(d.get_last("a"), Some(&text("3")));
    assert_eq!(d.get_first("c"), None);
    assert!(d.contains_key("b"));
}

#[test]
fn values_record_locations() {
    let value: Value = "{\n  name: Alice\n  pets [ fido ]\n}".parse().unwrap();
    assert_eq!(value.location(), Some(Location { line: 1, column: 1 }));
    let d = value.as_dictionary().unwrap();
    assert_eq!(
        d.get_first("name").unwrap().location(),
        Some(Location { line: 2, column: 9 })
    );
    assert_eq!(
        d.get_first("pets").unwrap().location(),
        Some(Location { line: 3, column: 8 })
    );
}

#[test]
fn equality_ignores_locations() {
    let parsed: Value = "{ a: [x, y], b: \"z\" }".parse().unwrap();
    let built: Dictionary = [
        ("a", Value::from(Array::from(vec![text("x"), text("y")]))),
        ("b", text("z")),
    ]
    .into_iter()
    .collect();
    assert_eq!(parsed, Value::from(built));
    assert_ne!(Text::new("z"), Text::new("y"));
}

#[test]
fn implicit_array_is_multidimensional() {
    let mut parser = Parser::new("a, b, c\nd\n\ne, f\n", "test");
    let a = parser.parse_implicit_array().unwrap().unwrap();
    assert_eq!(a.dimensions(), [2, 2, 3]);
    assert_eq!(a.get(&[0, 1, 0]), Some(&text("d")));
    assert_eq!(a.get(&[0, 1, 1]), Some(&text("")));
    assert_eq!(a.get(&[1, 0, 1]), Some(&text("f")));
    assert_eq!(a.get(&[2, 0, 0]), None);
    assert_eq!(a.get(&[0, 0]), None);
    assert_eq!(parser.parse_implicit_array().unwrap(), None);
}

#[test]
fn implicit_dictionary() {
    let mut parser = Parser::new("# config\nname: Alice\nage: 42\n", "test");
    let d = parser.parse_implicit_dictionary().unwrap().unwrap();
    assert_eq!(d.len(), 2);
    assert_eq!(d.get_first("age").and_then(Value::as_str), Some("42"));
}

#[test]
fn squeeze_removes_dimensions_of_size_one() {
    let column = Parser::new("a\nb\nc\n", "test")
        .parse_implicit_array()
        .unwrap()
        .unwrap();
    assert_eq!(column.dimensions(), [3, 1]);
    let squeezed = column.squeeze();
    assert_eq!(squeezed.dimensions(), [3]);
    assert_eq!(squeezed.elements(), column.elements());
    assert_eq!(Array::new(vec![2, 0], vec![]).squeeze().dimensions(), [0]);
}

#[test]
#[should_panic]
fn array_new_requires_matching_size() {
    Array::new(vec![2, 2], vec![text("a")]);
}

#[test]
fn zero_dimensional_array_is_written_without_a_comma() {
    let scalar = Value::from(Array::new(vec![], vec![text("x")]));
    assert_eq!(scalar.to_string(), "[\n  \"x\"\n]");
}

#[test]
fn display_round_trips() {
    let input = "{ \"a key\": [1, 2\n 3, 4], \": colon\": { x: \"line 1\nline 2\" }, e: [] }";
    let value: Value = input.parse().unwrap();
    let written = value.to_string();
    assert!(written.contains("a key: ["), "{written}");
    assert!(written.contains("\": colon\": {"), "{written}");
    assert_eq!(written.parse::<Value>().unwrap(), value);
}

#[test]
fn supplementary_characters_are_escaped_as_code_points() {
    let value: Value = "\"a\\u{1F600}b😀\"".parse().unwrap();
    assert_eq!(value.as_str(), Some("a\u{1F600}b\u{1F600}"));
    assert_eq!(value.to_string(), "\"a\\u{1F600}b\\u{1F600}\"");
}

#[test]
fn surrogate_pair_escapes_form_one_character() {
    let value: Value = "\"\\uD83D\\uDE00\"".parse().unwrap();
    assert_eq!(value.as_str(), Some("\u{1F600}"));
}

#[test]
fn unpaired_surrogate_escapes_are_errors() {
    for input in [
        "\"x\\uD83D\"",
        "\"x\\uD83D\\u0041\"",
        "\"x\\uD83D\\u{DE00}\"",
        "\"x\\uDE00\"",
        "\"x\\u{D83D}\"",
    ] {
        let e = input.parse::<Value>().unwrap_err();
        assert_eq!(
            e.location(),
            Some(Location { line: 1, column: 3 }),
            "{input}: {e}"
        );
    }
}

#[test]
fn from_str_requires_exactly_one_value() {
    let e = "".parse::<Value>().unwrap_err();
    assert_eq!(e.to_string(), "\"<string>\": No LWON value");
    let e = "[a]\n[b]".parse::<Value>().unwrap_err();
    assert_eq!(e.location(), Some(Location { line: 2, column: 1 }));
}

#[test]
fn errors_display_their_location() {
    let e = Parser::new("{ a: [1, 2 }", "f.lwon").parse().unwrap_err();
    assert_eq!(
        e.to_string(),
        "\"f.lwon\", line 1, char 6 ([): Missing closing ]"
    );
    assert_eq!(e.name(), Some("f.lwon"));
    assert_eq!(e.message(), "Missing closing ]");
}
