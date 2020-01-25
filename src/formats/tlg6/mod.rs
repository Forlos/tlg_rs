mod filter_types;
mod header;
mod line;
mod math;
mod pixels;
mod transformers;

use super::tlg6::{
    filter_types::Tlg6FilterTypes, header::TLG6Header, line::Tlg6Line, pixels::Pixels,
};
use crate::formats::constants::{TLG6_H_BLOCK_SIZE, TLG6_MAGIC, TLG6_W_BLOCK_SIZE, TLG_MAGIC_SIZE};
use crate::golomb::decode_golomb;

use image::{ImageBuffer, RgbaImage};
use scroll::{self, Pread, LE};
use std::{fs::File, io::Read};

#[derive(Debug)]
pub struct Tlg6 {
    header: TLG6Header,
    filter_types: Tlg6FilterTypes,
    lines: Vec<Tlg6Line>,
}

impl Tlg6 {
    /// Get new TLG6 instance from file
    pub fn from_file(file_name: &str) -> std::io::Result<Self> {
        let mut contents: Vec<u8> = Vec::with_capacity(1 << 20);
        File::open(file_name)?.read_to_end(&mut contents)?;
        if contents[0..TLG_MAGIC_SIZE] != TLG6_MAGIC {
            println!("Not a TLG file: {:?}", file_name);
            println!("Invalid magic: {:?}", &contents[0..TLG_MAGIC_SIZE]);
            // TODO handle this
            panic!();
        }
        Ok(Tlg6::from_bytes(&contents).unwrap())
    }
    /// Get new TLG6 instance from bytes
    pub fn from_bytes(buf: &[u8]) -> Result<Self, scroll::Error> {
        let offset = &mut TLG_MAGIC_SIZE;
        let header = buf.gread_with::<TLG6Header>(offset, LE)?;
        let filter_types = buf.gread_with::<Tlg6FilterTypes>(offset, LE)?;
        let lines_count = if header.image_height as usize % TLG6_H_BLOCK_SIZE == 0 {
            header.image_height as usize / TLG6_H_BLOCK_SIZE
        } else {
            header.image_height as usize / TLG6_H_BLOCK_SIZE + 1
        };
        let mut lines = Vec::with_capacity(lines_count);
        for _ in 0..lines_count {
            lines.push(buf.gread_with(offset, header.colors as usize)?)
        }
        Ok(Self {
            header,
            filter_types,
            lines,
        })
    }
    /// Convert TLG6 to RGBA image
    pub fn to_rgba_image(&self) -> Result<RgbaImage, scroll::Error> {
        let mut pixels = Pixels::new(
            4 * self.header.image_width as usize * self.header.image_height as usize,
            self.header.image_width as usize * 4,
        );
        let x_block_count = ((self.header.image_width as usize - 1) / TLG6_W_BLOCK_SIZE) + 1;
        let y_block_count = ((self.header.image_height as usize - 1) / TLG6_H_BLOCK_SIZE) + 1;
        let main_count = self.header.image_width / TLG6_W_BLOCK_SIZE as u32;
        let fraction = self.header.image_width - main_count * TLG6_W_BLOCK_SIZE as u32;
        let filter_types = self.filter_types.decompress(x_block_count * y_block_count);
        let mut pixel_buf: Vec<u8> =
            vec![0; 16 * self.header.image_width as usize * TLG6_H_BLOCK_SIZE];

        for (i, line) in self.lines.iter().enumerate() {
            let y = i * TLG6_H_BLOCK_SIZE;
            let mut y_lim = y + TLG6_H_BLOCK_SIZE;
            if y_lim > self.header.image_height as usize {
                y_lim = self.header.image_height as usize
            }
            let pixel_count = (y_lim - y) * self.header.image_width as usize;

            for (c, bits) in line.bits.iter().enumerate() {
                match bits.method {
                    0 => decode_golomb(&mut pixel_buf[c..], pixel_count, &bits.bit_pool)?,
                    _ => unimplemented!(),
                }
            }

            let ft = &filter_types[(y / TLG6_H_BLOCK_SIZE) * x_block_count..];
            let skip_bytes = (y_lim - y) * TLG6_W_BLOCK_SIZE;

            for yy in y..y_lim {
                let dir = (yy & 1) ^ 1;
                let odd_skip: i64 = (y_lim as i64 - yy as i64 - 1) - (yy as i64 - y as i64);

                if main_count != 0 {
                    let start = (if (self.header.image_width as usize) < TLG6_W_BLOCK_SIZE {
                        self.header.image_width as usize
                    } else {
                        TLG6_W_BLOCK_SIZE
                    }) * (yy - y);

                    pixels.decode_line(
                        self.header.image_width,
                        0,
                        main_count as usize,
                        ft,
                        skip_bytes,
                        &pixel_buf[start * 4..],
                        if self.header.colors == 3 {
                            0xFF000000
                        } else {
                            0
                        },
                        odd_skip,
                        dir,
                        self.header.colors,
                    )?;
                }

                if main_count != x_block_count as u32 {
                    let ww = if fraction as usize > TLG6_W_BLOCK_SIZE {
                        TLG6_W_BLOCK_SIZE
                    } else {
                        fraction as usize
                    };

                    let start = ww * (yy - y);

                    pixels.decode_line(
                        self.header.image_width,
                        main_count as usize,
                        x_block_count,
                        ft,
                        skip_bytes,
                        &pixel_buf[start * 4..],
                        if self.header.colors == 3 {
                            0xFF000000
                        } else {
                            0
                        },
                        odd_skip,
                        dir,
                        self.header.colors,
                    )?;
                }
                pixels.increment_offsets();
            }
        }

        Ok(ImageBuffer::from_raw(
            self.header.image_width,
            self.header.image_height,
            pixels.buf,
        )
        .unwrap())
    }
}
