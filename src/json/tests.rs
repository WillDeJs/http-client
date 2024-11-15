use crate::json::{JsonParser, JsonValue};

#[test]
fn parse_json_student_list() {
    let students = r#"{
        "class": "5th Grade Room B",
        "students": [
            {
                "name": "michael",
                "email": "mike@mail.com",
                "grades": [
                    90,
                    86,
                    93
                ]
            },
            {
                "name": "carl",
                "email": "carl@mail.com",
                "grades": [
                    82,
                    75,
                    79
                ]
            },
            {
                "name": "james",
                "email": "james@mail.com",
                "grades": [
                    80,
                    79,
                    89
                ]
            }
        ],
    }"#;

    let json = JsonParser::parse_json(&students).expect("Fail parsing student list");
    assert_eq!(
        json["class"],
        JsonValue::String("5th Grade Room B".to_owned())
    );
    assert_eq!(
        json["students"][0]["name"],
        JsonValue::String("michael".into())
    );
    assert_eq!(json["students"][0]["grades"][2].integer(), Some(&93));
    let james_grades = json["students"][2]["grades"].array().unwrap();
    assert_eq!(james_grades.iter().next(), Some(&JsonValue::Integer(80)));
    assert_eq!(james_grades.len(), 3);
}

#[test]
fn invalid_json() {
    let json_text = r#"{key": }"#;
    let json = JsonParser::parse_json(&json_text);
    assert!(json.is_err());

    let json_text = r#"{key: 25 }"#;
    let json = JsonParser::parse_json(&json_text);
    assert!(json.is_err());
}
