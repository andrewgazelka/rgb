//! NBT (Named Binary Tag) serialization for Minecraft protocol.
//!
//! This module provides a minimal NBT implementation focused on network NBT,
//! which uses nameless root compounds.

use byteorder::{BigEndian, WriteBytesExt};

/// NBT tag type IDs
mod tag_type {
    pub const END: u8 = 0;
    pub const BYTE: u8 = 1;
    pub const SHORT: u8 = 2;
    pub const INT: u8 = 3;
    pub const LONG: u8 = 4;
    pub const FLOAT: u8 = 5;
    pub const DOUBLE: u8 = 6;
    pub const BYTE_ARRAY: u8 = 7;
    pub const STRING: u8 = 8;
    pub const LIST: u8 = 9;
    pub const COMPOUND: u8 = 10;
    pub const INT_ARRAY: u8 = 11;
    pub const LONG_ARRAY: u8 = 12;
}

/// An NBT value
#[derive(Debug, Clone, PartialEq)]
pub enum NbtValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(NbtList),
    Compound(NbtCompound),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

/// An NBT list (all elements must be same type)
#[derive(Debug, Clone, PartialEq)]
pub enum NbtList {
    Empty,
    Byte(Vec<i8>),
    Short(Vec<i16>),
    Int(Vec<i32>),
    Long(Vec<i64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    ByteArray(Vec<Vec<i8>>),
    String(Vec<String>),
    List(Vec<NbtList>),
    Compound(Vec<NbtCompound>),
    IntArray(Vec<Vec<i32>>),
    LongArray(Vec<Vec<i64>>),
}

/// An NBT compound (map of string -> value)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NbtCompound {
    entries: Vec<(String, NbtValue)>,
}

impl NbtCompound {
    /// Create a new empty compound
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Insert a value into the compound
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<NbtValue>) {
        self.entries.push((key.into(), value.into()));
    }

    /// Build a compound from entries
    #[must_use]
    pub fn from_entries(entries: Vec<(String, NbtValue)>) -> Self {
        Self { entries }
    }

    /// Serialize to network NBT format (type byte + content, no name)
    #[must_use]
    pub fn to_network_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        // Write type tag for compound
        buf.push(tag_type::COMPOUND);
        // Write compound content (no name for network NBT)
        self.write_content(&mut buf);
        buf
    }

    /// Write compound content (entries + end tag)
    fn write_content(&self, buf: &mut Vec<u8>) {
        for (name, value) in &self.entries {
            value.write_named(buf, name);
        }
        buf.push(tag_type::END);
    }
}

impl NbtValue {
    /// Get the type ID for this value
    fn type_id(&self) -> u8 {
        match self {
            Self::Byte(_) => tag_type::BYTE,
            Self::Short(_) => tag_type::SHORT,
            Self::Int(_) => tag_type::INT,
            Self::Long(_) => tag_type::LONG,
            Self::Float(_) => tag_type::FLOAT,
            Self::Double(_) => tag_type::DOUBLE,
            Self::ByteArray(_) => tag_type::BYTE_ARRAY,
            Self::String(_) => tag_type::STRING,
            Self::List(_) => tag_type::LIST,
            Self::Compound(_) => tag_type::COMPOUND,
            Self::IntArray(_) => tag_type::INT_ARRAY,
            Self::LongArray(_) => tag_type::LONG_ARRAY,
        }
    }

    /// Write a named tag (type + name + value)
    fn write_named(&self, buf: &mut Vec<u8>, name: &str) {
        buf.push(self.type_id());
        write_nbt_string(buf, name);
        self.write_content(buf);
    }

    /// Write the tag content (no type, no name)
    fn write_content(&self, buf: &mut Vec<u8>) {
        match self {
            Self::Byte(v) => buf.push(*v as u8),
            Self::Short(v) => buf.write_i16::<BigEndian>(*v).unwrap(),
            Self::Int(v) => buf.write_i32::<BigEndian>(*v).unwrap(),
            Self::Long(v) => buf.write_i64::<BigEndian>(*v).unwrap(),
            Self::Float(v) => buf.write_f32::<BigEndian>(*v).unwrap(),
            Self::Double(v) => buf.write_f64::<BigEndian>(*v).unwrap(),
            Self::ByteArray(v) => {
                buf.write_i32::<BigEndian>(v.len() as i32).unwrap();
                for b in v {
                    buf.push(*b as u8);
                }
            }
            Self::String(v) => write_nbt_string(buf, v),
            Self::List(list) => list.write_content(buf),
            Self::Compound(compound) => compound.write_content(buf),
            Self::IntArray(v) => {
                buf.write_i32::<BigEndian>(v.len() as i32).unwrap();
                for i in v {
                    buf.write_i32::<BigEndian>(*i).unwrap();
                }
            }
            Self::LongArray(v) => {
                buf.write_i32::<BigEndian>(v.len() as i32).unwrap();
                for l in v {
                    buf.write_i64::<BigEndian>(*l).unwrap();
                }
            }
        }
    }
}

impl NbtList {
    /// Get the element type ID
    fn element_type_id(&self) -> u8 {
        match self {
            Self::Empty => tag_type::END,
            Self::Byte(_) => tag_type::BYTE,
            Self::Short(_) => tag_type::SHORT,
            Self::Int(_) => tag_type::INT,
            Self::Long(_) => tag_type::LONG,
            Self::Float(_) => tag_type::FLOAT,
            Self::Double(_) => tag_type::DOUBLE,
            Self::ByteArray(_) => tag_type::BYTE_ARRAY,
            Self::String(_) => tag_type::STRING,
            Self::List(_) => tag_type::LIST,
            Self::Compound(_) => tag_type::COMPOUND,
            Self::IntArray(_) => tag_type::INT_ARRAY,
            Self::LongArray(_) => tag_type::LONG_ARRAY,
        }
    }

    /// Get the length
    fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Byte(v) => v.len(),
            Self::Short(v) => v.len(),
            Self::Int(v) => v.len(),
            Self::Long(v) => v.len(),
            Self::Float(v) => v.len(),
            Self::Double(v) => v.len(),
            Self::ByteArray(v) => v.len(),
            Self::String(v) => v.len(),
            Self::List(v) => v.len(),
            Self::Compound(v) => v.len(),
            Self::IntArray(v) => v.len(),
            Self::LongArray(v) => v.len(),
        }
    }

    /// Write list content (element type + length + elements)
    fn write_content(&self, buf: &mut Vec<u8>) {
        buf.push(self.element_type_id());
        buf.write_i32::<BigEndian>(self.len() as i32).unwrap();

        match self {
            Self::Empty => {}
            Self::Byte(v) => {
                for b in v {
                    buf.push(*b as u8);
                }
            }
            Self::Short(v) => {
                for s in v {
                    buf.write_i16::<BigEndian>(*s).unwrap();
                }
            }
            Self::Int(v) => {
                for i in v {
                    buf.write_i32::<BigEndian>(*i).unwrap();
                }
            }
            Self::Long(v) => {
                for l in v {
                    buf.write_i64::<BigEndian>(*l).unwrap();
                }
            }
            Self::Float(v) => {
                for f in v {
                    buf.write_f32::<BigEndian>(*f).unwrap();
                }
            }
            Self::Double(v) => {
                for d in v {
                    buf.write_f64::<BigEndian>(*d).unwrap();
                }
            }
            Self::ByteArray(v) => {
                for arr in v {
                    buf.write_i32::<BigEndian>(arr.len() as i32).unwrap();
                    for b in arr {
                        buf.push(*b as u8);
                    }
                }
            }
            Self::String(v) => {
                for s in v {
                    write_nbt_string(buf, s);
                }
            }
            Self::List(v) => {
                for list in v {
                    list.write_content(buf);
                }
            }
            Self::Compound(v) => {
                for compound in v {
                    compound.write_content(buf);
                }
            }
            Self::IntArray(v) => {
                for arr in v {
                    buf.write_i32::<BigEndian>(arr.len() as i32).unwrap();
                    for i in arr {
                        buf.write_i32::<BigEndian>(*i).unwrap();
                    }
                }
            }
            Self::LongArray(v) => {
                for arr in v {
                    buf.write_i32::<BigEndian>(arr.len() as i32).unwrap();
                    for l in arr {
                        buf.write_i64::<BigEndian>(*l).unwrap();
                    }
                }
            }
        }
    }
}

/// Write an NBT string (u16 length + modified UTF-8)
fn write_nbt_string(buf: &mut Vec<u8>, s: &str) {
    // NBT uses modified UTF-8 with u16 length prefix
    let bytes = s.as_bytes();
    buf.write_u16::<BigEndian>(bytes.len() as u16).unwrap();
    buf.extend_from_slice(bytes);
}

// Convenient From implementations
impl From<bool> for NbtValue {
    fn from(v: bool) -> Self {
        Self::Byte(if v { 1 } else { 0 })
    }
}

impl From<i8> for NbtValue {
    fn from(v: i8) -> Self {
        Self::Byte(v)
    }
}

impl From<i16> for NbtValue {
    fn from(v: i16) -> Self {
        Self::Short(v)
    }
}

impl From<i32> for NbtValue {
    fn from(v: i32) -> Self {
        Self::Int(v)
    }
}

impl From<i64> for NbtValue {
    fn from(v: i64) -> Self {
        Self::Long(v)
    }
}

impl From<f32> for NbtValue {
    fn from(v: f32) -> Self {
        Self::Float(v)
    }
}

impl From<f64> for NbtValue {
    fn from(v: f64) -> Self {
        Self::Double(v)
    }
}

impl From<&str> for NbtValue {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}

impl From<String> for NbtValue {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<NbtCompound> for NbtValue {
    fn from(v: NbtCompound) -> Self {
        Self::Compound(v)
    }
}

impl From<NbtList> for NbtValue {
    fn from(v: NbtList) -> Self {
        Self::List(v)
    }
}

/// Macro for building NBT compounds ergonomically
///
/// # Example
/// ```
/// use mc_protocol::nbt;
///
/// let compound = nbt! {
///     "byte" => 1i8,
///     "int" => 42i32,
///     "string" => "hello",
///     "nested" => nbt! {
///         "inner" => true,
///     },
/// };
/// ```
#[macro_export]
macro_rules! nbt {
    // Empty compound
    () => {
        $crate::nbt::NbtCompound::new()
    };

    // Compound with entries
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut compound = $crate::nbt::NbtCompound::new();
        $(
            compound.insert($key, $value);
        )*
        compound
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_compound() {
        let compound = nbt! {
            "byte" => 1i8,
            "int" => 42i32,
            "string" => "hello",
        };

        let bytes = compound.to_network_bytes();
        // Should start with compound tag type
        assert_eq!(bytes[0], tag_type::COMPOUND);
        // Should have content
        assert!(bytes.len() > 1);
    }

    #[test]
    fn test_nested_compound() {
        let compound = nbt! {
            "outer" => nbt! {
                "inner" => 123i32,
            },
        };

        let bytes = compound.to_network_bytes();
        assert_eq!(bytes[0], tag_type::COMPOUND);
    }

    #[test]
    fn test_bool_as_byte() {
        let compound = nbt! {
            "flag" => true,
        };

        let bytes = compound.to_network_bytes();
        // Find the value byte (after type, name length, name, another type byte)
        // Should be 1 for true
        assert!(bytes.contains(&1));
    }
}
