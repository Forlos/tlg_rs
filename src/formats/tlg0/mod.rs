use super::tlg6;
use crate::formats::constants::{TLG5_MAGIC, TLG6_MAGIC, TLG_MAGIC_SIZE};

use image::RgbaImage;
use scroll::{self, ctx, Pread, LE};
use std::{fs::File, io::Read};

#[derive(Debug)]
pub struct Tlg0 {
    raw_length: u32,
    raw_data: TlgRawData,
    //TODO add support for tlg0 chunks
}

#[derive(Debug)]
enum TlgRawData {
    Tlg6 { data: tlg6::Tlg6 },
}

impl<'a> ctx::TryFromCtx<'a, usize> for TlgRawData {
    type Error = failure::Error;
    fn try_from_ctx(buf: &'a [u8], _: usize) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let mut magic = [0; TLG_MAGIC_SIZE];
        buf.take(TLG_MAGIC_SIZE as u64).read_exact(&mut magic)?;
        let data = match magic {
            TLG5_MAGIC => todo!(),
            TLG6_MAGIC => TlgRawData::Tlg6 {
                data: tlg6::Tlg6::from_bytes(buf)?,
            },
            _ => unimplemented!(),
        };
        Ok((data, *offset as usize))
    }
}

impl Tlg0 {
    pub fn from_file(file_name: &str) -> Result<Self, failure::Error> {
        let mut contents: Vec<u8> = Vec::with_capacity(1 << 20);
        File::open(file_name)?.read_to_end(&mut contents)?;
        Ok(Tlg0::from_bytes(&contents)?)
    }
    pub fn from_bytes(buf: &[u8]) -> Result<Self, failure::Error> {
        let offset = &mut TLG_MAGIC_SIZE;
        let raw_length = buf.gread_with::<u32>(offset, LE)?;
        let raw_data = buf.gread_with::<TlgRawData>(offset, raw_length as usize)?;
        Ok(Self {
            raw_length,
            raw_data,
        })
    }
    pub fn to_rgba_image(&self) -> Result<RgbaImage, failure::Error> {
        match &self.raw_data {
            TlgRawData::Tlg6 { data } => data.to_rgba_image(),
        }
    }
}
