use std::{collections::HashMap, iter::Peekable, str::ParseBoolError};

use super::{JsonObj, JsonValue};

/// A simple JSON Parser
/// # Example:
/// ```
/// use http_client::json::*;
/// fn main() {
///     let text = r#"{"name":"Mike", "age": 23, "email": null, "grades": [90, 89, 79]}"#;
///     let json_object = JsonParser::parse_json(text).unwrap();
///     assert_eq!(json_object["age"].integer(), Some(&23));
///     assert_eq!(json_object["email"], JsonValue::Null);
///     assert_eq!(json_object["grades"][0].integer(), Some(&90));
/// }
/// ```
pub struct JsonParser;

impl JsonParser {
    /// Parse a JSON value from the given string.
    pub fn parse_json(value: &str) -> Result<JsonValue, String> {
        Self::parse_item(&mut value.chars().peekable())
    }
    fn parse_item<I>(data: &mut Peekable<I>) -> Result<JsonValue, String>
    where
        I: Iterator<Item = char>,
    {
        // skip whitespace
        Self::skip_whitespace(data);

        if let Some(c) = data.peek() {
            match c {
                '"' => Self::parse_string(data),
                '0'..='9' | '-' => Self::parse_number(data),
                't' | 'f' => Self::parse_boolean(data),
                '[' => Self::parse_array(data),
                'n' => Self::parse_null(data),
                '{' => Self::parse_object(data),
                _ => Err(format!("Unexpected character `{c}` found in JSON Object")),
            }
        } else {
            Err("Cannot parse empty object.".to_string())
        }
    }

    fn parse_string<I>(data: &mut Peekable<I>) -> Result<JsonValue, String>
    where
        I: Iterator<Item = char>,
    {
        let mut result = String::new();
        data.next(); // skip quote
        loop {
            let c = data.peek();
            match c {
                Some('\\') => match data.next() {
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some('/') => result.push('/'),
                    Some('t') => result.push('/'),
                    Some('f') => result.push('\u{008}'),
                    Some('b') => result.push('\u{00C}'),
                    Some('n') => result.push('\n'),
                    Some(other) => {
                        return Err(format!("Invalid escape sequence in data `\\{other}` "))
                    }
                    _ => return Err(format!("Incomplete escape sequence in Json Object.")),
                },
                Some('"') => break,
                Some(other) => result.push(*other),
                _ => return Err(format!("Incomplete String value found `{result}`")),
            };
            data.next(); // move to the next location
        }
        data.next();
        Ok(JsonValue::String(result))
    }
    fn parse_number<I>(data: &mut Peekable<I>) -> Result<JsonValue, String>
    where
        I: Iterator<Item = char>,
    {
        let mut number_data = String::new();
        while let Some(c) = data.peek() {
            if *c == '-' || *c == '.' || *c == 'e' || *c == 'E' || c.is_numeric() {
                number_data.push(*c);
            } else {
                break;
            }
            data.next();
        }
        if number_data.contains('.') {
            match number_data.parse::<f64>() {
                Ok(value) => Ok(JsonValue::Float(value)),
                Err(_) => Err(format!("Could not parse number value `{number_data}`")),
            }
        } else {
            match number_data.parse::<isize>() {
                Ok(value) => Ok(JsonValue::Integer(value)),
                Err(_) => Err(format!("Could not parse number value `{number_data}`")),
            }
        }
    }
    fn parse_boolean<I>(data: &mut Peekable<I>) -> Result<JsonValue, String>
    where
        I: Iterator<Item = char>,
    {
        let boolean_data = data.take(4).collect::<String>();
        let boolean_value: bool = boolean_data
            .parse()
            .map_err(|e: ParseBoolError| e.to_string())?;
        Ok(JsonValue::Boolean(boolean_value))
    }
    fn parse_array<I>(data: &mut Peekable<I>) -> Result<JsonValue, String>
    where
        I: Iterator<Item = char>,
    {
        let mut array = Vec::new();
        // skip opening bracket
        data.next();
        // skip whitespace
        loop {
            Self::skip_whitespace(data);
            match data.peek() {
                Some(']') => {
                    data.next();
                    break;
                }
                Some(',') => {
                    data.next();
                    Self::skip_whitespace(data);
                }
                Some(_) => {
                    let value = Self::parse_item(data)?;
                    array.push(value);
                }
                None => return Err(format!("Could not parse complete array from given values")),
            };
        }
        Ok(JsonValue::Array(array))
    }
    fn parse_null<I>(data: &mut Peekable<I>) -> Result<JsonValue, String>
    where
        I: Iterator<Item = char>,
    {
        let null_data = data.take(4).collect::<String>();
        if null_data == "null" {
            Ok(JsonValue::Null)
        } else {
            Err(format!("Cannot build JSON value from `{null_data}`"))
        }
    }
    fn parse_object<I>(data: &mut Peekable<I>) -> Result<JsonValue, String>
    where
        I: Iterator<Item = char>,
    {
        let mut map = HashMap::new();
        // skip opening bracket
        data.next();
        // skip whitespace
        loop {
            Self::skip_whitespace(data);
            match data.peek() {
                Some('}') => {
                    data.next();
                    break;
                }
                Some(',') => {
                    data.next();
                    Self::skip_whitespace(data);
                }
                Some(_) => {
                    Self::skip_whitespace(data);
                    let key = match Self::parse_item(data)? {
                        JsonValue::String(value) => value,
                        _ => return Err(format!("Expected String key for object.")),
                    };
                    Self::skip_whitespace(data);
                    if Some(':') != data.next() {
                        return Err(format!("Incomplete object. Expected `:` after key `{key}"));
                    }

                    Self::skip_whitespace(data);
                    let value = Self::parse_item(data)?;
                    map.insert(key, value);
                }

                None => return Err(format!("Could not parse complete object from given values")),
            };
        }
        Ok(JsonValue::Object(JsonObj { inner: map }))
    }
    fn skip_whitespace<I>(data: &mut Peekable<I>)
    where
        I: Iterator<Item = char>,
    {
        loop {
            match data.peek() {
                Some(c) if c.is_whitespace() => {
                    data.next();
                }
                _ => break,
            }
        }
    }
}
