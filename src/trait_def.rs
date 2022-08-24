use std::borrow::Cow;

use crate::{Error, DB};

// We actually want to split the Codec into an Encode and a Decode trait,
// but I have merged them here for simplicity
pub trait Codec<'a> {
    type Item: 'a;
    type Error;

    fn encode(item: Self::Item) -> Result<Cow<'a, [u8]>, Self::Error>;
    fn decode(bytes: &'a [u8]) -> Result<Self::Item, Self::Error>;
}

// I don't think this will actually be useful for us
pub trait DefaultCodec<'a> {
    type Codec: Codec<'a>;
}

// here is what it looks like once split:
pub trait Encode<'a> {
    type EItem: 'a;
    type Error;

    fn decode(bytes: &'a [u8]) -> Result<Self::EItem, Self::Error>;
}
pub trait Decode<'a> {
    type DItem: 'a;
    type Error;

    fn decode(bytes: &'a [u8]) -> Result<Self::DItem, Self::Error>;
}

// Then we can have a convenience trait merging the two:
pub trait Codec2<'a>: Encode<'a, EItem = Self::Item> + Decode<'a, DItem = Self::Item> {
    type Item: 'a;
}

// when the lifetime doesn't matter:
// note that this trait needs to be implemented manually, just like Codec2, Encode, and Decode
pub trait CodecOwned: for<'a> Codec2<'a> {
    type OwnedItem;
}

// now let's see what the function signatures look like:
fn get_with_codec<'t, K, V>(db: &'t DB, key: K::Item) -> Result<V, Error>
where
    K: Codec<'t>,
    V: Codec<'t>,
{
    todo!()
}

fn get_with_codec_split<'t, K, V>(db: &'t DB, key: <K as Encode<'t>>::EItem) -> Result<V, Error>
where
    K: Encode<'t> + Decode<'t, DItem = <K as Encode<'t>>::EItem>,
    V: Encode<'t> + Decode<'t, DItem = <V as Encode<'t>>::EItem>,
{
    todo!()
}

fn get_with_codec2<'t, K, V>(db: &'t DB, key: K::Item) -> Result<V, Error>
where
    K: Codec2<'t>,
    V: Codec2<'t>,
{
    todo!()
}

// I don't really see the point of it tbh
fn get_with_codec_owned<'t, K, V>(db: &'t DB, key: &K::OwnedItem) -> Result<V, Error>
where
    K: CodecOwned,
    V: CodecOwned,
{
    todo!()
}
