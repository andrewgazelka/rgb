use std::borrow::Cow;
use std::io::{self, Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "derive")]
pub use mc_protocol_derive::{Decode, Encode};

// Re-export serde for use by generated code
pub use serde;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("VarInt too large")]
    VarIntTooLarge,
    #[error("String too long: {len} > {max}")]
    StringTooLong { len: usize, max: usize },
    #[error("Invalid enum variant: {0}")]
    InvalidEnumVariant(i32),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, ProtocolError>;

/// Protocol state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum State {
    Handshaking,
    Status,
    Login,
    Configuration,
    Play,
}

/// Packet direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Clientbound,
    Serverbound,
}

/// Trait for all packets - provides ID, name, state, and direction
pub trait Packet {
    /// The packet ID
    const ID: i32;
    /// The packet name (e.g., "MovePlayerPos")
    const NAME: &'static str;
    /// The protocol state this packet belongs to
    const STATE: State;
    /// Whether this packet is clientbound or serverbound
    const DIRECTION: Direction;
}

pub trait Encode {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()>;
}

pub trait Decode<'a>: Sized {
    fn decode<R: Read>(reader: &mut R) -> Result<Self>;
}

// VarInt encoding/decoding
pub fn read_varint<R: Read>(reader: &mut R) -> Result<i32> {
    let mut result = 0i32;
    let mut shift = 0;
    loop {
        let byte = reader.read_u8()?;
        result |= ((byte & 0x7F) as i32) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 32 {
            return Err(ProtocolError::VarIntTooLarge);
        }
    }
    Ok(result)
}

pub fn write_varint<W: Write>(writer: &mut W, mut value: i32) -> Result<()> {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value = ((value as u32) >> 7) as i32;
        if value != 0 {
            byte |= 0x80;
        }
        writer.write_u8(byte)?;
        if value == 0 {
            break;
        }
    }
    Ok(())
}

// VarLong encoding/decoding
pub fn read_varlong<R: Read>(reader: &mut R) -> Result<i64> {
    let mut result = 0i64;
    let mut shift = 0;
    loop {
        let byte = reader.read_u8()?;
        result |= ((byte & 0x7F) as i64) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            return Err(ProtocolError::VarIntTooLarge);
        }
    }
    Ok(result)
}

pub fn write_varlong<W: Write>(writer: &mut W, mut value: i64) -> Result<()> {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value = ((value as u64) >> 7) as i64;
        if value != 0 {
            byte |= 0x80;
        }
        writer.write_u8(byte)?;
        if value == 0 {
            break;
        }
    }
    Ok(())
}

// Primitive implementations
impl Encode for bool {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u8(if *self { 1 } else { 0 })?;
        Ok(())
    }
}

impl Decode<'_> for bool {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_u8()? != 0)
    }
}

impl Encode for u8 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u8(*self)?;
        Ok(())
    }
}

impl Decode<'_> for u8 {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_u8()?)
    }
}

impl Encode for i8 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_i8(*self)?;
        Ok(())
    }
}

impl Decode<'_> for i8 {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_i8()?)
    }
}

impl Encode for i16 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_i16::<BigEndian>(*self)?;
        Ok(())
    }
}

impl Decode<'_> for i16 {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_i16::<BigEndian>()?)
    }
}

impl Encode for u16 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u16::<BigEndian>(*self)?;
        Ok(())
    }
}

impl Decode<'_> for u16 {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_u16::<BigEndian>()?)
    }
}

impl Encode for i32 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_i32::<BigEndian>(*self)?;
        Ok(())
    }
}

impl Decode<'_> for i32 {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_i32::<BigEndian>()?)
    }
}

impl Encode for i64 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_i64::<BigEndian>(*self)?;
        Ok(())
    }
}

impl Decode<'_> for i64 {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_i64::<BigEndian>()?)
    }
}

impl Encode for f32 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_f32::<BigEndian>(*self)?;
        Ok(())
    }
}

impl Decode<'_> for f32 {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_f32::<BigEndian>()?)
    }
}

impl Encode for f64 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_f64::<BigEndian>(*self)?;
        Ok(())
    }
}

impl Decode<'_> for f64 {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_f64::<BigEndian>()?)
    }
}

// VarInt wrapper type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VarInt(pub i32);

impl Encode for VarInt {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        write_varint(writer, self.0)
    }
}

impl Decode<'_> for VarInt {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(VarInt(read_varint(reader)?))
    }
}

impl From<i32> for VarInt {
    fn from(v: i32) -> Self {
        VarInt(v)
    }
}

impl From<VarInt> for i32 {
    fn from(v: VarInt) -> Self {
        v.0
    }
}

// VarLong wrapper type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VarLong(pub i64);

impl Encode for VarLong {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        write_varlong(writer, self.0)
    }
}

impl Decode<'_> for VarLong {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(VarLong(read_varlong(reader)?))
    }
}

// String encoding (length-prefixed with VarInt)
impl Encode for str {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        let bytes = self.as_bytes();
        write_varint(writer, bytes.len() as i32)?;
        writer.write_all(bytes)?;
        Ok(())
    }
}

impl Encode for String {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.as_str().encode(writer)
    }
}

impl Decode<'_> for String {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let len = read_varint(reader)? as usize;
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;
        Ok(String::from_utf8(buf)?)
    }
}

impl<'a> Encode for Cow<'a, str> {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.as_ref().encode(writer)
    }
}

// Option<T> encoding (bool prefix)
impl<T: Encode> Encode for Option<T> {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        match self {
            Some(v) => {
                true.encode(writer)?;
                v.encode(writer)
            }
            None => false.encode(writer),
        }
    }
}

impl<'a, T: Decode<'a>> Decode<'a> for Option<T> {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        if bool::decode(reader)? {
            Ok(Some(T::decode(reader)?))
        } else {
            Ok(None)
        }
    }
}

// Vec<T> encoding (VarInt length prefix)
impl<T: Encode> Encode for Vec<T> {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        write_varint(writer, self.len() as i32)?;
        for item in self {
            item.encode(writer)?;
        }
        Ok(())
    }
}

impl<'a, T: Decode<'a>> Decode<'a> for Vec<T> {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let len = read_varint(reader)? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::decode(reader)?);
        }
        Ok(vec)
    }
}

// UUID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Uuid(pub u128);

impl Encode for Uuid {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u64::<BigEndian>((self.0 >> 64) as u64)?;
        writer.write_u64::<BigEndian>(self.0 as u64)?;
        Ok(())
    }
}

impl Decode<'_> for Uuid {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let high = reader.read_u64::<BigEndian>()? as u128;
        let low = reader.read_u64::<BigEndian>()? as u128;
        Ok(Uuid((high << 64) | low))
    }
}

// Position (packed x/y/z)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i16,
    pub z: i32,
}

// NBT placeholder (raw bytes for now)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Nbt(pub Vec<u8>);

// BlockState placeholder (VarInt encoded)
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct BlockState(pub i32);

impl Encode for Nbt {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.0)?;
        Ok(())
    }
}

impl Decode<'_> for Nbt {
    fn decode<R: Read>(_reader: &mut R) -> Result<Self> {
        // NBT decoding is complex - for now just return empty
        // TODO: implement proper NBT parsing
        Ok(Nbt(Vec::new()))
    }
}

impl Encode for BlockState {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        write_varint(writer, self.0)
    }
}

impl Decode<'_> for BlockState {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(BlockState(read_varint(reader)?))
    }
}

impl Encode for Position {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        let packed = ((self.x as i64 & 0x3FFFFFF) << 38)
            | ((self.z as i64 & 0x3FFFFFF) << 12)
            | (self.y as i64 & 0xFFF);
        writer.write_i64::<BigEndian>(packed)?;
        Ok(())
    }
}

impl Decode<'_> for Position {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let packed = reader.read_i64::<BigEndian>()?;
        let x = (packed >> 38) as i32;
        let y = (packed << 52 >> 52) as i16;
        let z = (packed << 26 >> 38) as i32;
        Ok(Position { x, y, z })
    }
}
