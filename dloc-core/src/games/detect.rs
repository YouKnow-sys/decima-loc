//! Functions to try to detect what game is the input core file from

use std::io::{Read, Seek, SeekFrom};

use binrw::BinRead;

use crate::{
    games::{ds, hzd},
    DResult, Error,
};

/// An enum representing the different games that can be detected.
///
/// Variants:
///
/// - `Hzd`: Represents Horizon Zero Dawn.
/// - `Ds`: Represents Death Stranding.
/// - `Mixed`: Used when multiple games are detected.
/// - `Unknown`: Used when no known game is detected.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GameDetection {
    /// Horizon Zero Dawn
    Hzd,
    /// Death Stranding
    Ds,
    /// This should never happen, but its better to be in the safe side
    Mixed,
    /// No supported game
    Unknown,
}

/// Detects which game a core file is from by checking magic numbers.
///
/// Reads magic numbers from the core file. Increments counters when known magic numbers are found.
/// At the end, checks the counter to determine which game it likely is.
///
/// Returns a [`GameDetection`] enum variant indicating which game was detected,
/// [`GameDetection::Mixed`] if multiple games were detected, or [`GameDetection::Unknown`] if no known games were detected.
pub fn detect_game<R: Read + Seek>(reader: &mut R) -> DResult<GameDetection> {
    let mut hzd = 0_usize;
    let mut ds = 0_usize;

    loop {
        let magic = match u64::read_le(reader) {
            Ok(m) => m,
            Err(e) => {
                if e.is_eof() {
                    break;
                } else {
                    return Err(Error::BinRw(e));
                }
            }
        };

        let buf_size = u32::read_le(reader)?;

        match magic {
            hzd::LOCALIZED_MAGIC | hzd::CUTSCENE_MAGIC => hzd += 1,
            ds::LOCALIZED_MAGIC => ds += 1,
            _ => (),
        }

        reader.seek(SeekFrom::Current(buf_size as i64))?;
    }

    Ok(match (hzd.eq(&0), ds.eq(&0)) {
        (true, true) => GameDetection::Unknown,
        (false, true) => GameDetection::Hzd,
        (true, false) => GameDetection::Ds,
        (false, false) => GameDetection::Mixed,
    })
}
