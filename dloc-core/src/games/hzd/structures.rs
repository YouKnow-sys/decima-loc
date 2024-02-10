use std::{fmt::Debug, mem::size_of};

use binrw::binrw;

use crate::{
    games::chunks::RuntimeSize,
    utils::{
        types::{U16String, U8String},
        EnumKey,
    },
};

use super::{FixedMap, Language};

pub const LOCALIZED_MAGIC: u64 = 0xB89A596B420BB2E2;
pub const CUTSCENE_MAGIC: u64 = 0x5A3ECD4ADA693D7F;

pub type Chunk = crate::games::chunks::Chunk<ChunkVariants>;

/// Represents the possible variants for a Chunk, which contains either
/// localized text data, cutscene data, or unknown data. The variant is
/// determined by the Chunk's magic number and assertions are used to
/// validate the magic number matches the variant.
#[binrw]
#[brw(little)]
#[br(import(magic: u64, size: u32))]
#[derive(Hash)]
pub enum ChunkVariants {
    #[br(pre_assert(magic == LOCALIZED_MAGIC))]
    Localized(Box<Localized>),
    #[br(pre_assert(magic == CUTSCENE_MAGIC))]
    Cutscene(Box<Cutscene>),
    /// Data variant for unknown chunk data.
    /// Stores raw binary data.
    Others {
        #[br(count = size, err_context("Invalid core file, size = {}", size))]
        data: Vec<u8>,
    },
}

impl ChunkVariants {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Localized(_) => "Localized",
            Self::Cutscene(_) => "Cutscene",
            Self::Others { .. } => "Others",
        }
    }
}

impl RuntimeSize for ChunkVariants {
    fn rt_size(&self) -> u32 {
        match self {
            Self::Localized(loc) => loc.rt_size(),
            Self::Cutscene(cut) => cut.rt_size(),
            Self::Others { data } => data.len() as u32,
        }
    }
}

impl Debug for ChunkVariants {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Localized(arg) => f.debug_tuple("Localized").field(arg).finish(),
            Self::Cutscene(arg) => f.debug_tuple("Cutscene").field(arg).finish(),
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
    pub strings: FixedMap<U8String>,
}

impl RuntimeSize for Localized {
    fn rt_size(&self) -> u32 {
        (self.uuid.len()
            + self
                .strings
                .iter()
                .map(|(_, s)| s.full_size())
                .sum::<usize>()) as u32
    }
}

impl Debug for Localized {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Localized")
            .field("uuid", &format!("{:#x?}", self.uuid))
            .field("strings", &self.strings)
            .finish()
    }
}

#[binrw]
#[brw(little)]
#[derive(Hash)]
pub struct Cutscene {
    uuid: [u8; 16],
    useless_block_len: u32,
    #[br(count = useless_block_len + 4)]
    useless_block: Vec<u8>,
    #[br(assert(lang_count == Language::LEN as u32, "Language count doesn't match with what dloc expect HZD to have {lang_count} != {}.", Language::LEN))]
    lang_count: u32,
    #[br(map = sort_cutscene_group)]
    pub list: FixedMap<CutsceneStringGroup>,
    unk: [u8; 5],
}

impl RuntimeSize for Cutscene {
    fn rt_size(&self) -> u32 {
        let other_sizes = self.uuid.len()
            + (self.useless_block_len as usize + 4) // u32 + usless_block
            + (size_of::<u32>() * 2) // 4 extra bytes after useless block + lang_count
            + self.unk.len();

        let string_sizes: usize = self
            .list
            .iter()
            .map(|(_, l)| {
                (size_of::<u32>() * 2) // lang_code + count
                    + l.strings_data
                        .iter()
                        .map(|s| s.string.full_size() + size_of::<u64>())
                        .sum::<usize>()
            })
            .sum();
        (other_sizes + string_sizes) as u32
    }
}

impl Debug for Cutscene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cutscene")
            .field("uuid", &format!("{:#x?}", self.uuid))
            .field("useless_block_len", &self.useless_block_len)
            .field("useless_block", &self.useless_block)
            .field("lang_count", &self.lang_count)
            .field("list", &self.list)
            .field("unk", &format!("{:#x?}", self.unk))
            .finish()
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Hash)]
pub struct CutsceneStringGroup {
    #[br(assert(lang_code <= Language::LEN as _, "Invalid core file, lang code was \"{lang_code}\". it shouldn't be bigger then {}", Language::LEN))]
    lang_code: u32,
    count: u32,
    #[br(count = count)]
    pub strings_data: Vec<CutsceneStringData>,
}

#[binrw]
#[brw(little)]
#[derive(Clone, Hash)]
pub struct CutsceneStringData {
    pub string: U16String,
    timing: u64,
}

impl Debug for CutsceneStringData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CutsceneStringData")
            .field("string", &self.string)
            .field("timing", &format!("{:#x?}", self.timing))
            .finish()
    }
}

/// Sorts the cutscene string groups in the given map by their language code.
/// Returns the sorted map.
fn sort_cutscene_group(mut map: FixedMap<CutsceneStringGroup>) -> FixedMap<CutsceneStringGroup> {
    map.inner.sort_by(|e1, e2| e1.lang_code.cmp(&e2.lang_code));
    map
}
