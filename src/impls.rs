use std::{borrow::Cow, marker::PhantomData};

use crate::{
    trait_def::{Codec, DefaultCodec},
    Error,
};

pub struct ByteSlice;
impl<'a> Codec<'a> for ByteSlice {
    type Item = &'a [u8];
    type Error = Error;
    fn encode(item: Self::Item) -> Result<Cow<'a, [u8]>, Self::Error> {
        Ok(Cow::Borrowed(item))
    }
    fn decode(bytes: &'a [u8]) -> Result<Self::Item, Self::Error> {
        Ok(bytes)
    }
}
impl<'a> DefaultCodec<'a> for &'a [u8] {
    type Codec = ByteSlice;
}

pub struct U8Codec;
impl<'a> Codec<'a> for U8Codec {
    type Item = u8;
    type Error = Error;
    fn encode(item: Self::Item) -> Result<Cow<'a, [u8]>, Self::Error> {
        Ok(Cow::Owned(vec![item]))
    }
    fn decode(bytes: &'a [u8]) -> Result<Self::Item, Self::Error> {
        Ok(bytes[0])
    }
}
impl<'a> DefaultCodec<'a> for u8 {
    type Codec = ByteMuckCodec<u8>;
}

/// A Codec that can encode/decode any type implementing bytemuck::Pod
pub struct ByteMuckCodec<T> {
    _phantom: PhantomData<T>,
}
impl<'a, T> Codec<'a> for ByteMuckCodec<T>
where
    T: bytemuck::Pod,
{
    type Item = &'a T;
    type Error = Error;

    fn encode(item: Self::Item) -> Result<Cow<'a, [u8]>, Self::Error> {
        Ok(Cow::Borrowed(bytemuck::bytes_of(item)))
    }

    fn decode(bytes: &'a [u8]) -> Result<Self::Item, Self::Error> {
        Ok(bytemuck::from_bytes(bytes))
    }
}

// It is also possible to encode/decode a type that has references in it
// via the Codec trait
#[derive(Clone, Copy)]
pub struct MyStructRef<'a> {
    pub x: &'a str,
    pub y: &'a str,
}
pub struct MyStructOwned {
    pub x: String,
    pub y: String,
}

// much simpler than the mess above:
pub enum MyStruct<'a> {
    Ref(MyStructRef<'a>),
    Owned(MyStructOwned),
}
impl<'a> MyStruct<'a> {
    fn get_ref<'b: 'a>(&'b self) -> MyStructRef<'a> {
        match self {
            MyStruct::Ref(x) => *x,
            MyStruct::Owned(o) => MyStructRef {
                x: o.x.as_str(),
                y: o.y.as_str(),
            },
        }
    }
}

struct MyStructCodec;
impl<'a> Codec<'a> for MyStructCodec {
    type Item = MyStruct<'a>;
    type Error = Error;

    fn encode(item: Self::Item) -> Result<Cow<'a, [u8]>, Self::Error> {
        // This is verbose but unavoidable
        let MyStructRef { x, y } = item.get_ref();
        let mut bytes = vec![0];
        bytes.extend_from_slice(x.as_bytes());
        bytes.extend_from_slice(y.as_bytes());
        Ok(Cow::Owned(bytes))
    }

    fn decode(bytes: &'a [u8]) -> Result<Self::Item, Self::Error> {
        // I can't tell here in what case we want to have a ref and in what case we want a owned,
        // so instead I simulate that condition by reading the first byte
        // in real life, we'd decide based on the alignment of the byte array, I guess
        let is_ref = bytes[0] == 0;
        if is_ref {
            let x = std::str::from_utf8(&bytes[1..=4]).unwrap();
            let y = std::str::from_utf8(&bytes[5..=8]).unwrap();
            Ok(MyStruct::Ref(MyStructRef { x, y }))
        } else {
            let x = std::str::from_utf8(&bytes[1..=4]).unwrap().to_owned();
            let y = std::str::from_utf8(&bytes[5..=8]).unwrap().to_owned();
            Ok(MyStruct::Owned(MyStructOwned { x, y }))
        }
    }
}

// generic version of the above:

// This trait does not actually work very well
// we probably want GAT instead, see gat.rs for a version
// of this trait with GAT
pub trait MyRef<'a> {
    type Ref;
    fn get_ref(&'a self) -> Self::Ref;
    fn to_owned(reference: Self::Ref) -> Self;
}

impl<'a> MyRef<'a> for MyStructOwned {
    type Ref = MyStructRef<'a>;

    fn get_ref(&'a self) -> Self::Ref {
        MyStructRef {
            x: self.x.as_str(),
            y: self.y.as_str(),
        }
    }

    fn to_owned(reference: Self::Ref) -> Self {
        MyStructOwned {
            x: reference.x.to_owned(),
            y: reference.y.to_owned(),
        }
    }
}

// Honestly such a generic type isn't needed,
// I prefer the specific type approach
pub enum RefOrOwned<'a, T>
where
    T: MyRef<'a>,
{
    Ref(<T as MyRef<'a>>::Ref),
    Owned(T),
}
