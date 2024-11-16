use std::{
    collections::{hash_map::Iter, HashMap},
    fmt::Display,
    ops::Index,
};

/// A struct that holds a JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Float(f64),
    Integer(isize),
    Boolean(bool),
    String(String),
    Array(Vec<JsonValue>),
    Object(JsonObj),
}

impl JsonValue {
    /// Get a value from inside this JSON value if it exists inside this JSON Object
    /// # Arguments
    /// `key`   key name to be retrieved
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(json_obj) => json_obj.get(key),
            _ => None,
        }
    }

    /// Parse this JSON Object as a `f64` is possible.
    pub fn float(&self) -> Option<&f64> {
        match self {
            JsonValue::Float(value) => Some(value),
            _ => None,
        }
    }
    /// Parse this JSON Object as a `isize` is possible.
    pub fn integer(&self) -> Option<&isize> {
        match self {
            JsonValue::Integer(value) => Some(value),
            _ => None,
        }
    }
    /// Parse this JSON Object as a `bool` is possible.
    pub fn boolean(&self) -> Option<&bool> {
        match self {
            JsonValue::Boolean(value) => Some(value),
            _ => None,
        }
    }
    /// Parse this JSON Object as a `Vec<JsonValue` is possible.
    pub fn array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(value) => Some(value),
            _ => None,
        }
    }
}

/// A JSON Object struct.
/// Holds a Map of other JSON Values.
///
/// # Example:
/// ```
/// use http_client::json::*;
/// fn main() {
///     let mut json_object = JsonObj::new();
///     json_object.insert("name", "Michael");
///     json_object.insert("age", 27);
///     json_object.insert("email", Option::<&str>::None);
///     assert_eq!(json_object["age"].integer(), Some(&27));
///     assert_eq!(json_object["email"], JsonValue::Null);
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct JsonObj {
    pub(crate) inner: HashMap<String, JsonValue>,
}
impl JsonObj {
    /// Create an empty JSON Object
    pub fn new() -> JsonObj {
        let inner = HashMap::new();
        JsonObj { inner }
    }
    /// Insert any value that can be turned into a JSON Value inside this Object
    ///
    /// # Arguments:
    /// `key`   Key name being inserted.
    /// `value` Value being inserted.
    pub fn insert<T>(&mut self, key: &str, value: T) -> bool
    where
        JsonValue: From<T>,
    {
        self.inner.insert(key.to_owned(), value.into()).is_some()
    }

    /// Get a value  from this JSON Object (If any) using the given given Key.
    ///
    /// # Arguments:
    /// `key`   Key name being retrieved.
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        self.inner.get(key)
    }

    /// Get's an iterator of all the key, value pairs inside this JSON Object Map.
    pub fn iter(&self) -> Iter<'_, String, JsonValue> {
        self.inner.iter()
    }
}

impl Index<&str> for JsonObj {
    type Output = JsonValue;

    fn index(&self, index: &str) -> &Self::Output {
        &self.inner[index]
    }
}
impl Index<&str> for JsonValue {
    type Output = JsonValue;

    fn index(&self, index: &str) -> &Self::Output {
        match self {
            JsonValue::Object(json_obj) => &json_obj[index],
            _ => panic!("Indexing into this kind of JSON value not allowed"),
        }
    }
}
impl Index<usize> for JsonValue {
    type Output = JsonValue;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            JsonValue::Array(array) => &array[index],
            _ => panic!("Indexing into this kind of JSON value not allowed"),
        }
    }
}

/// Display Implementations
impl Display for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Float(v) => write!(f, "{}", v),
            JsonValue::Integer(v) => write!(f, "{}", v),
            JsonValue::Boolean(v) => write!(f, "{}", v),
            JsonValue::String(v) => write!(f, "\"{}\"", v),
            JsonValue::Array(vec) => {
                write!(f, "[")?;
                for i in 0..vec.len().saturating_sub(1) {
                    write!(f, "{},", vec[i])?;
                }
                if vec.len() > 1 {
                    write!(f, "{}", vec[vec.len().saturating_sub(1)])?;
                }
                write!(f, "]")
            }
            JsonValue::Object(hash_map) => {
                write!(f, "{}", "{")?;
                let last = hash_map.inner.len().saturating_sub(1);
                for (index, (key, value)) in hash_map.inner.iter().enumerate() {
                    write!(f, "\"{}\": {}", key, value)?;
                    if index < last {
                        write!(f, ",")?;
                    }
                }
                write!(f, "{}", "}")?;
                Ok(())
            }
        }
    }
}

impl Display for JsonObj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "{")?;
        let last = self.inner.len() - 1;
        for (index, (key, value)) in self.inner.iter().enumerate() {
            write!(f, "\"{}\": {}", key, value)?;
            if index < last {
                write!(f, ",")?;
            }
        }
        write!(f, "{}", "}")
    }
}

/// Conversion implementations
impl From<&String> for JsonValue {
    fn from(value: &String) -> Self {
        JsonValue::String(value.to_owned())
    }
}
impl From<&str> for JsonValue {
    fn from(value: &str) -> Self {
        JsonValue::String(value.to_owned())
    }
}
impl From<isize> for JsonValue {
    fn from(value: isize) -> Self {
        JsonValue::Integer(value)
    }
}
impl From<i32> for JsonValue {
    fn from(value: i32) -> Self {
        JsonValue::Integer(value as isize)
    }
}
impl From<i16> for JsonValue {
    fn from(value: i16) -> Self {
        JsonValue::Integer(value as isize)
    }
}
impl From<i8> for JsonValue {
    fn from(value: i8) -> Self {
        JsonValue::Integer(value as isize)
    }
}
impl From<f32> for JsonValue {
    fn from(value: f32) -> Self {
        JsonValue::Float(value as f64)
    }
}
impl From<f64> for JsonValue {
    fn from(value: f64) -> Self {
        JsonValue::Float(value)
    }
}
impl From<bool> for JsonValue {
    fn from(value: bool) -> Self {
        JsonValue::Boolean(value)
    }
}
impl From<JsonObj> for JsonValue {
    fn from(value: JsonObj) -> Self {
        JsonValue::Object(value)
    }
}
impl<T> From<Option<T>> for JsonValue
where
    JsonValue: From<T>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => JsonValue::from(v),
            None => JsonValue::Null,
        }
    }
}

impl<T> From<&Vec<T>> for JsonValue
where
    JsonValue: From<T>,
    T: Clone,
{
    fn from(value: &Vec<T>) -> Self {
        let vec = value.iter().map(|v| JsonValue::from(v.clone())).collect();
        JsonValue::Array(vec)
    }
}
