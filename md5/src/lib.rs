//! The [MD5][1] hash function.
//!
//! [1]: https://en.wikipedia.org/wiki/MD5

#![no_std]
extern crate generic_array;
extern crate byte_tools;
extern crate digest;
extern crate digest_buffer;
#[cfg(feature = "asm")]
extern crate md5_asm as utils;

#[cfg(not(feature = "asm"))]
mod utils;

use utils::compress;

pub use digest::Digest;
use byte_tools::{write_u32_le, write_u32v_le};
use digest_buffer::{DigestBuffer};
use generic_array::GenericArray;
use generic_array::typenum::{U16, U64};

mod consts;
use consts::S0;

type BlockSize = U64;
type Block = GenericArray<u8, BlockSize>;

/// The MD5 Digest algorithm
#[derive(Copy, Clone)]
pub struct Md5 {
    length_bytes: u64,
    buffer: DigestBuffer<BlockSize>,
    state: [u32; 4],
}

impl Default for Md5 {
    fn default() -> Self {
        Md5 {
            length_bytes: 0,
            buffer: Default::default(),
            state: S0,
        }
    }
}

impl Md5 {
    fn finalize(&mut self) {
        let self_state = &mut self.state;
        self.buffer.standard_padding(8, |d: &Block| {
            compress(self_state, d);
        });
        write_u32_le(self.buffer.next(4), (self.length_bytes << 3) as u32);
        write_u32_le(self.buffer.next(4), (self.length_bytes >> 29) as u32);
        compress(self_state, self.buffer.full_buffer());
    }
}

impl digest::Input for Md5 {
    type BlockSize = BlockSize;

    fn digest(&mut self, input: &[u8]) {
        // Unlike Sha1 and Sha2, the length value in MD5 is defined as
        // the length of the message mod 2^64 - ie: integer overflow is OK.
        self.length_bytes += input.len() as u64;
        let self_state = &mut self.state;
        self.buffer.input(input, |d: &Block| {
            compress(self_state, d);
        });
    }
}

impl digest::FixedOutput for Md5 {
    type OutputSize = U16;

    fn fixed_result(mut self) -> GenericArray<u8, Self::OutputSize> {
        self.finalize();

        let mut out = GenericArray::default();
        write_u32v_le(&mut out, &self.state);
        out
    }
}
