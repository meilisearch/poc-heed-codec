use std::borrow::Cow;

use crate::{
    impls::{MyStructOwned, MyStructRef},
    Error, DB,
};

// This file explores how the traits could change if we could
// use generic associated types

pub trait CodecGAT {
    type Item<'a>;
    type Error;

    fn encode<'a>(item: Self::Item<'a>) -> Result<Cow<'a, [u8]>, Self::Error>;
    fn decode<'a>(bytes: &'a [u8]) -> Result<Self::Item<'a>, Self::Error>;
}

// it seems like it makes the bounds better
fn get_with_codec<'t, K, V>(db: &'t DB, key: K::Item<'t>) -> Result<V, Error>
where
    K: CodecGAT,
    V: CodecGAT,
{
    todo!()
}
// except that if we split the Codec trait into two:
pub trait EncodeGAT {
    type EItem<'a>;
    type Error;

    fn encode<'a>(item: Self::EItem<'a>) -> Result<Cow<'a, [u8]>, Self::Error>;
}
pub trait DecodeGAT {
    type DItem<'a>;
    type Error;

    fn decode<'a>(bytes: &'a [u8]) -> Result<Self::DItem<'a>, Self::Error>;
}
// then the bounds get difficult as well:
fn get_with_codec_2<'t, K, V>(db: &'t DB, key: <K as EncodeGAT>::EItem<'t>) -> Result<V, Error>
where
    // but in many cases we don't actually want an equivalence between EItem and DItem
    for<'a> K: EncodeGAT + DecodeGAT<DItem<'a> = <K as EncodeGAT>::EItem<'a>>,
    for<'a> V: EncodeGAT + DecodeGAT<DItem<'a> = <V as EncodeGAT>::EItem<'a>>,
{
    todo!()
}
// but we can also split the Codec type into Encode and Decode and then redefine it as:
trait CodecGAT2:
    for<'a> EncodeGAT<EItem<'a> = Self::Item<'a>> + for<'a> DecodeGAT<DItem<'a> = Self::Item<'a>>
{
    type Item<'a>;
}

// then we're okay:
fn get_with_codec_3<'t, K, V>(db: &'t DB, key: K::Item<'t>) -> Result<V, Error>
where
    K: CodecGAT2,
    V: CodecGAT2,
{
    todo!()
}
// but it's also possible without GAT, and I don't think that's what we want either

// in short, GATs make the API more logical, but they don't really solve a lot of hard problems
// for the Codec traits

// On the other hand, the `MyRef` trait we had in `impls.rs` would be much nicer with GAT
// That is not very important since I don't think we wanted this trait anyway

pub trait MyRefGAT {
    // Ideally I'd be able to say that Ref<'a> is covariant over 'a, but that is not possible
    // https://internals.rust-lang.org/t/variance-of-lifetime-arguments-in-gats/14769/17
    type Ref<'a>: 'a + Copy + Sized;
    // so instead we require the user of the trait to specify the casting function manually
    // the correct implementation simply returns `r`
    fn upcast<'a: 'b, 'b>(r: Self::Ref<'a>) -> Self::Ref<'b>;

    fn get_ref<'a>(&'a self) -> Self::Ref<'a>;
    fn to_owned(reference: Self::Ref<'_>) -> Self;
}

pub enum RefOrOwnedGAT<'a, T>
where
    T: MyRefGAT,
{
    Ref(<T as MyRefGAT>::Ref<'a>),
    Owned(T),
}

impl<'a, T> RefOrOwnedGAT<'a, T>
where
    T: MyRefGAT,
    for<'c> <T as MyRefGAT>::Ref<'c>: Copy,
{
    fn get_ref<'b>(&'b self) -> <T as MyRefGAT>::Ref<'b>
    where
        'a: 'b,
    {
        match self {
            RefOrOwnedGAT::Ref(x) => T::upcast(*x),
            RefOrOwnedGAT::Owned(_) => todo!(),
        }
    }
}

impl MyRefGAT for MyStructOwned {
    type Ref<'a> = MyStructRef<'a>;

    fn upcast<'a: 'b, 'b>(r: Self::Ref<'a>) -> Self::Ref<'b> {
        r
    }

    fn get_ref<'a>(&'a self) -> Self::Ref<'a> {
        MyStructRef {
            x: self.x.as_str(),
            y: self.y.as_str(),
        }
    }

    fn to_owned(reference: Self::Ref<'_>) -> Self {
        MyStructOwned {
            x: reference.x.to_owned(),
            y: reference.y.to_owned(),
        }
    }
}

struct MyStructCodec3;
impl CodecGAT for MyStructCodec3 {
    type Item<'a> = RefOrOwnedGAT<'a, MyStructOwned>;
    type Error = Error;

    fn encode<'a>(item: Self::Item<'a>) -> Result<Cow<'a, [u8]>, Self::Error> {
        let MyStructRef { x, y } = item.get_ref();
        let mut bytes = vec![0];
        bytes.extend_from_slice(x.as_bytes());
        bytes.extend_from_slice(y.as_bytes());
        Ok(Cow::Owned(bytes))
    }

    fn decode<'a>(bytes: &'a [u8]) -> Result<Self::Item<'a>, Self::Error> {
        // I can't tell here in what case we want to have a ref and in what case we want a owned,
        // so instead I simulate that condition by reading the first byte
        let is_ref = bytes[0] == 0;
        if is_ref {
            let x = std::str::from_utf8(&bytes[1..=4]).unwrap();
            let y = std::str::from_utf8(&bytes[5..=8]).unwrap();
            Ok(RefOrOwnedGAT::Ref(MyStructRef { x, y }))
        } else {
            let x = std::str::from_utf8(&bytes[1..=4]).unwrap().to_owned();
            let y = std::str::from_utf8(&bytes[5..=8]).unwrap().to_owned();
            Ok(RefOrOwnedGAT::Owned(MyStructOwned { x, y }))
        }
    }
}
