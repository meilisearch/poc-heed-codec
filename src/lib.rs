// I explore how GAT could change the traits in gat.rs
#![feature(generic_associated_types)]

pub mod gat;
pub mod impls;
pub mod trait_def;

// Dummy error type
#[derive(Clone, Copy, Debug)]
pub struct Error;

// this type is not actually used for anything, but I might use it
// later to write some examples
#[derive(Default)]
pub struct DB {
    pub keys: Vec<Vec<u8>>,
    pub values: Vec<Vec<u8>>,
}
