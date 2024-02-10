use binrw::{binrw, BinRead, BinWrite};

/// Represents a chunk of binary data with a magic number, size, and variant
/// payload. Used for serialized game data.
#[binrw]
#[brw(little)]
#[derive(Debug, Hash)]
pub struct Chunk<V>
where
    for<'a> V: RuntimeSize + BinRead<Args<'a> = (u64, u32)> + BinWrite<Args<'a> = ()>,
{
    magic: u64,
    #[br(temp)]
    #[bw(calc = variant.rt_size())]
    size: u32,
    #[br(args(magic, size))]
    pub variant: V,
}

/// A helper trait to get the size of a object in runtime
pub trait RuntimeSize {
    /// size of the whole variant
    fn rt_size(&self) -> u32;
}
