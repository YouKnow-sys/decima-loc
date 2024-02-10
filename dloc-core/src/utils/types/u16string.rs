use std::{
    fmt::Display,
    io::{Read, Seek},
    mem::size_of,
    ops::{Deref, DerefMut},
};

use binrw::{args, BinRead, BinResult, BinWrite};

/// A string type that stores a UTF-16 encoded string with a 32-bit length prefix.
///
/// Implements [`BinRead`] and [`BinWrite`] for serialization, as well as [`Deref`]
/// and [`DerefMut`] for ergonomic access to the inner [`String`].
#[derive(Debug, Clone, Hash)]
pub struct U16String(String);

impl U16String {
    /// Returns the total size in bytes of the string when serialized.
    /// This includes the 4 byte length prefix and the UTF-16 encoded string.
    pub fn full_size(&self) -> usize {
        size_of::<u32>() + (self.u16_len() * size_of::<u16>())
    }

    /// Returns the length of the string in UTF-16 code units.
    // So bad, maybe we could instead store a `Vec<u16>` to the Self instead of string?
    // but then we can't check for errors until we try to convert it to string...
    pub fn u16_len(&self) -> usize {
        self.0.encode_utf16().count()
    }
}

impl BinRead for U16String {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        _: Self::Args<'_>,
    ) -> BinResult<Self> {
        let len = u32::read_options(reader, endian, ())?;
        let bytes = <Vec<u16>>::read_options(reader, endian, args! { count: len as usize })?;

        let string = String::from_utf16(&bytes).map_err(|e| binrw::Error::Custom {
            pos: reader.stream_position().unwrap_or_default(),
            err: Box::new(e),
        })?;

        Ok(Self(string))
    }
}

impl BinWrite for U16String {
    type Args<'a> = ();

    fn write_options<W: std::io::prelude::Write + Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        _: Self::Args<'_>,
    ) -> BinResult<()> {
        if self.0.is_empty() {
            // if string is empty just write size
            return 0_u32.write_options(writer, endian, ());
        }

        let u16str: Vec<u16> = self.0.encode_utf16().collect();

        (u16str.len() as u32).write_options(writer, endian, ())?;
        u16str.write_options(writer, endian, ())
    }
}

impl Display for U16String {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<U16String> for String {
    fn from(val: U16String) -> Self {
        val.0
    }
}

impl From<String> for U16String {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Deref for U16String {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for U16String {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
