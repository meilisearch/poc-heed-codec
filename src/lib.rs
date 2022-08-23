use std::borrow::Cow;
use std::ops::RangeBounds;

// Those are fake types just to express what we really
// want to use in the final heed version.

struct Txn;

struct Error;

struct RangeIterator<'t, T>(&'t (), T);

// --------

// I would like to get rid of these two traits, by rewriting them
// directly on the types to serialize/deserialize themselves.

trait BytesEncode<'a> {
    type EItem: ?Sized + 'a;

    fn bytes_encode(item: &'a Self::EItem) -> Result<Cow<'a, [u8]>, Error>;

    // I would also like to expose these two additional methods that would let
    // the implementor to just write into the provided buffer when the size is known.
    //
    // see http://www.lmdb.tech/doc/group__mdb.html#ga32a193c6bf4d7d5c5d579e71f22e9340 (section MDB_NOMEMINIT)
    // and http://www.lmdb.tech/doc/group__mdb__put.html#gac0545c6aea719991e3eae6ccc686efcc (section MDB_RESERVE)

    fn serialized_size(item: &'a Self::EItem) -> Option<usize> {
        None
    }

    fn bytes_encode_in_slice(
        item: &'a Self::EItem,
        output: &mut [u8],
    ) -> Result<Cow<'a, [u8]>, Error> {
        todo!()
    }
}

// The big issue with this type is that GAT doens't exist and therefore we can't
// express DItem<'a> but DItem: 'a that forces us to expose the lifetime 'a onto
// the codec itself.
//
// The codec should not be aware of the lifetime as it is stateless
// and all the methods are static (not tacking self).

trait BytesDecode<'a> {
    type DItem: 'a;

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, Error>;
}

// --------

// All these function must define that K and V are bounded by the lifetime 't.

fn put<'t, K, V>(txn: &'t Txn, key: K, value: V) -> Result<(), Error>
where
    K: BytesEncode,
    V: BytesEncode,
{
    todo!()
}

fn get<'t, K, V>(txn: &'t Txn, key: K) -> Result<V, Error>
where
    K: BytesEncode,
    V: BytesDecode,
{
    todo!()
}

// what is funny is that RangeBounds accepts a T: ?Sized.
// https://doc.rust-lang.org/nightly/std/ops/trait.RangeBounds.html
fn get_range<'t, R, K, V>(txn: &'t Txn, range: K) -> Result<RangeIterator<'t, (K, V)>, Error>
where
    R: RangeBounds<K>,
    K: BytesEncode + BytesDecode,
    V: BytesDecode,
{
    todo!()
}

fn main() {
    let txn = Txn;
}
