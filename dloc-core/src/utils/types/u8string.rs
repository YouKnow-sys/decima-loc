use std::{
    fmt::Display,
    io::{Read, Seek},
    mem::size_of,
    ops::{Deref, DerefMut},
};

use binrw::{BinRead, BinResult, BinWrite};

/// A string type that stores a UTF-8 encoded string with a 16-bit length prefix.
///
/// Implements [`BinRead`] and [`BinWrite`] for serialization, as well as [`Deref`]
/// and [`DerefMut`] for ergonomic access to the inner [`String`].
#[derive(Debug, Clone, Hash)]
pub struct U8String(String);

impl U8String {
    /// Get the full size of the underlying buffer
    pub fn full_size(&self) -> usize {
        size_of::<u16>() + self.len()
    }
}

impl BinRead for U8String {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        _: Self::Args<'_>,
    ) -> BinResult<Self> {
        let len = u16::read_options(reader, endian, ())?;
        let mut buf = vec![0_u8; len as usize];
        reader.read_exact(&mut buf)?;

        let string = String::from_utf8(buf).map_err(|e| binrw::Error::Custom {
            pos: reader.stream_position().unwrap_or_default(),
            err: Box::new(e),
        })?;

        Ok(Self(string))
    }
}

impl BinWrite for U8String {
    type Args<'a> = ();

    fn write_options<W: std::io::prelude::Write + Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        _: Self::Args<'_>,
    ) -> BinResult<()> {
        if self.0.is_empty() {
            // if string is empty just write size
            return 0_u16.write_options(writer, endian, ());
        }

        (self.0.len() as u16).write_options(writer, endian, ())?;
        self.0.as_bytes().write_options(writer, endian, ())
    }
}

impl Display for U8String {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<U8String> for String {
    fn from(val: U8String) -> Self {
        val.0
    }
}

impl From<String> for U8String {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Deref for U8String {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for U8String {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
