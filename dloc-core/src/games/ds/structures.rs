use std::{fmt::Debug, mem::size_of};

use binrw::binrw;

use super::FixedMap;
use crate::{games::chunks::RuntimeSize, utils::types::U8String};

pub const LOCALIZED_MAGIC: u64 = 0x31BE502435317445;

pub type Chunk = crate::games::chunks::Chunk<ChunkVariants>;

#[binrw]
#[brw(little)]
#[br(import(magic: u64, size: u32))]
#[derive(Hash)]
pub enum ChunkVariants {
    #[br(pre_assert(magic == LOCALIZED_MAGIC))]
    Localized(Box<Localized>),
    /// Data variant for unknown chunk data.
    /// Stores raw binary data.
    Others {
        #[br(count = size, err_context("Invalid core file, size = {}", size))]
        data: Vec<u8>,
    },
}

impl RuntimeSize for ChunkVariants {
    fn rt_size(&self) -> u32 {
        match self {
            Self::Localized(loc) => loc.rt_size(),
            Self::Others { data } => data.len() as u32,
        }
    }
}

impl Debug for ChunkVariants {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Localized(arg) => f.debug_tuple("Localized").field(arg).finish(),
            Self::Others { data } => f
                .debug_struct("Others")
                .field("raw_data_bytes", &data.len())
                .finish(),
        }
    }
}

#[binrw]
#[brw(little)]
#[derive(Hash)]
pub struct Localized {
    uuid: [u8; 16],
    pub string_groups: FixedMap<LocalGroup>,
}

impl RuntimeSize for Localized {
    fn rt_size(&self) -> u32 {
        self.uuid.len() as u32
            + self
                .string_groups
                .inner
                .iter()
                .map(|g| g.rt_size())
                .sum::<u32>()
    }
}

impl Debug for Localized {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Localized")
            .field("uuid", &format!("{:#x?}", self.uuid))
            .field("string-groups", &self.string_groups)
            .finish()
    }
}

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Hash)]
pub struct LocalGroup {
    pub text: U8String,
    note: U8String,
    mode: u8,
}

impl RuntimeSize for LocalGroup {
    fn rt_size(&self) -> u32 {
        (self.text.full_size() + self.note.full_size() + size_of::<u8>()) as u32
    }
}
