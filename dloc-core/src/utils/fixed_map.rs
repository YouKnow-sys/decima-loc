use std::{
    array::IntoIter,
    fmt::Debug,
    io::{Read, Seek, Write},
    iter::Zip,
    marker::PhantomData,
    ops,
};

use binrw::{BinRead, BinResult, BinWrite};

pub trait EnumKey: From<usize> + Into<usize> {
    const LEN: usize;
}

pub trait Array {
    type List;
}

/// A fixed size map implementation that guarantees every possible key exists.
/// Uses an enum as the key type to provide a finite key space.
/// The map stores values in an array indexed by the enum keys.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedMap<const N: usize, K: EnumKey, V> {
    pub(crate) inner: [V; N],
    _phantom: PhantomData<K>,
}

impl<const N: usize, K: EnumKey, V> FixedMap<N, K, V> {
    /// Returns an iterator over the key-value pairs in the map.
    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        std::array::from_fn::<_, N, _>(|i| (K::from(i), &self.inner[i])).into_iter()
    }

    /// Map the inner value of the `FixedMap` to another type.
    pub(crate) fn map_inner<B, F>(self, f: F) -> FixedMap<N, K, B>
    where
        F: FnMut(V) -> B,
        B: Debug,
    {
        let inner = self.inner.map(f);
        FixedMap {
            inner,
            _phantom: PhantomData,
        }
    }

    /// Returns the number of elements in the map.
    pub const fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn get(&self, key: K) -> &V {
        &self.inner[key.into()]
    }

    pub fn get_mut(&mut self, key: K) -> &mut V {
        &mut self.inner[key.into()]
    }
}

impl<const N: usize, K: EnumKey, V> ops::Index<K> for FixedMap<N, K, V> {
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        self.get(index)
    }
}

impl<const N: usize, K: EnumKey, V> ops::IndexMut<K> for FixedMap<N, K, V> {
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        self.get_mut(index)
    }
}

impl<const N: usize, K, V> BinRead for FixedMap<N, K, V>
where
    K: EnumKey,
    V: BinRead,
    for<'a> V::Args<'a>: Clone,
{
    type Args<'a> = V::Args<'a>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let inner = <[V; N]>::read_options(reader, endian, args)?;

        Ok(Self {
            inner,
            _phantom: PhantomData,
        })
    }
}

impl<const N: usize, K, V> BinWrite for FixedMap<N, K, V>
where
    K: EnumKey,
    V: BinWrite + 'static,
    for<'a> V::Args<'a>: Clone,
{
    type Args<'a> = V::Args<'a>;

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        self.inner.write_options(writer, endian, args)
    }
}

impl<const N: usize, K, V> Serialize for FixedMap<N, K, V>
where
    K: EnumKey + Serialize,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_map(self.iter())
    }
}

impl<'de, const N: usize, K, V> Deserialize<'de> for FixedMap<N, K, V>
where
    K: EnumKey + Deserialize<'de>,
    V: Default + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct FixedMapVisitor<const N: usize, K: EnumKey, V: Default>(PhantomData<(K, V)>);

        impl<'de, const N: usize, K: EnumKey, V: Default> Visitor<'de> for FixedMapVisitor<N, K, V>
        where
            K: EnumKey + Deserialize<'de>,
            V: Default + Deserialize<'de>,
        {
            type Value = FixedMap<N, K, V>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a fixed map of length {}", K::LEN)
            }

            fn visit_map<A>(self, mut entries: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut map = FixedMap::<N, K, V> {
                    inner: std::array::from_fn(|_| V::default()),
                    _phantom: PhantomData,
                };

                while let Some((key, value)) = entries.next_entry()? {
                    map[key] = value;
                }

                Ok(map)
            }
        }

        let visitor = FixedMapVisitor(PhantomData);
        deserializer.deserialize_map(visitor)
    }
}

impl<const N: usize, K, V> IntoIterator for FixedMap<N, K, V>
where
    K: EnumKey + Debug,
    V: Debug,
{
    type Item = (K, V);

    type IntoIter = Zip<IntoIter<K, N>, IntoIter<V, N>>;

    fn into_iter(self) -> Self::IntoIter {
        // as our map is Fixed we can do this without any problem
        std::array::from_fn(|i| K::from(i))
            .into_iter()
            .zip(self.inner)
    }
}

impl<const N: usize, K, V> Debug for FixedMap<N, K, V>
where
    K: EnumKey + Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

/// Count the number of input elements
macro_rules! count {
    () => (0_usize);
    ($x:tt $($xs:tt)*) => (1_usize + $crate::utils::count!($($xs)*));
}

pub(crate) use count;

/// A helper macro to generate a Enum map.
macro_rules! enum_map {
    ($(#[doc = $comment:literal])? $(#[derive($($derive_name:ident),+)])? $name:ident; $($variant_name:ident = $idx:literal),+ $(,)?) => {
        $(#[doc = $comment])?
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, $($($derive_name),+)?)]
        #[repr(u8)] // its all built-in so Im sure I nver need anything bigger then this...
        pub enum $name {
            $($variant_name = $idx),+
        }

        impl $name {
            pub const ALL_VARIANTS: [Self; $crate::utils::count!($($variant_name)*)] = [$(Self::$variant_name),+];
        }

        impl From<usize> for $name {
            fn from(value: usize) -> Self {
                match value {
                    $($idx => Self::$variant_name),+,
                    _ => unreachable!(),
                }
            }
        }

        // not a good way, but hey it works
        impl TryFrom<String> for $name {
            type Error = ();

            fn try_from(value: String) -> Result<Self, Self::Error> {
                let value = value.to_lowercase();

                $(if value == stringify!($variant_name).to_lowercase() {
                    return Ok(Self::$variant_name);
                })+

                Err(())
            }
        }

        impl From<$name> for usize {
            fn from(value: $name) -> Self {
                value as usize
            }
        }

        impl EnumKey for $name {
            const LEN: usize = $crate::utils::count!($($idx)*);
        }

        type FixedMap<V> = $crate::utils::FixedMap<{$name::LEN}, $name, V>;
    };
}

pub(crate) use enum_map;
use serde::{de::Visitor, Deserialize, Serialize};

#[cfg(test)]
mod test {
    use std::usize;

    use super::*;

    enum_map!(
        Key;

        One = 0,
        Two = 1,
        Three = 2,
        Four = 3,
        Five = 4,
    );

    const MAP: FixedMap<i32> = FixedMap {
        inner: [1, 2, 3, 4, 5],
        _phantom: PhantomData,
    };

    #[test]
    fn get_key() {
        assert_eq!(1, MAP[Key::One]);
        assert_eq!(3, MAP[Key::Three]);
        assert_eq!(5, MAP[Key::Five]);
    }

    #[test]
    fn update_value() {
        let mut map = MAP.clone();
        assert_eq!(1, map[Key::One]);
        map[Key::One] = 10;
        assert_eq!(10, map[Key::One]);
    }

    #[test]
    fn iter() {
        let mut iter = MAP.into_iter();

        assert_eq!(Some((Key::One, 1)), iter.next());
        assert_eq!(Some((Key::Two, 2)), iter.next());
        assert_eq!(Some((Key::Three, 3)), iter.next());
        assert_eq!(Some((Key::Four, 4)), iter.next());
        assert_eq!(Some((Key::Five, 5)), iter.next());
        assert_eq!(None, iter.next());

        assert_eq!(15, MAP.iter().map(|(_, v)| *v).sum::<i32>());
    }
}
