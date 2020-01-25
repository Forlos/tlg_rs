use super::{
    super::constants::TLG6_W_BLOCK_SIZE,
    math::{avg, med},
    transformers::transform,
};
use scroll::{self, Pread, Pwrite, LE};

#[derive(Debug)]
pub(crate) struct Pixels {
    pub(crate) buf: Vec<u8>,
    cur_index: usize,
    prev_index: usize,
    difference: usize,
}

impl Pixels {
    pub(crate) fn new(size: usize, difference: usize) -> Self {
        Self {
            buf: vec![0u8; size],
            cur_index: 0,
            prev_index: 0,
            difference,
        }
    }

    pub(crate) fn increment_offsets(&mut self) {
        self.cur_index += self.difference;
        self.prev_index = self.cur_index - self.difference;
    }

    pub(crate) fn get_cur_line(&mut self) -> &mut [u8] {
        &mut self.buf[self.cur_index..]
    }

    pub(crate) fn get_prev_line(&self) -> &[u8] {
        &self.buf[self.prev_index..]
    }

    pub(crate) fn decode_line(
        &mut self,
        width: u32,
        start_block: usize,
        block_limit: usize,
        filter_types: &[u8],
        skip_block_bytes: usize,
        pixel_buf: &[u8],
        initialp: u32,
        odd_skip: i64,
        dir: usize,
        colors: u8,
    ) -> Result<(), scroll::Error> {
        let mut p = initialp;
        let mut up = p;

        let mut prev_line_index = 0;
        let mut cur_line_index = 0;
        let mut pixel_buf_index = 0;

        if start_block != 0 {
            prev_line_index += start_block * TLG6_W_BLOCK_SIZE * 4;
            cur_line_index += start_block * TLG6_W_BLOCK_SIZE * 4;
            up = self
                .get_prev_line()
                .pread_with::<u32>(prev_line_index - 1, LE)?;
            p = self
                .get_cur_line()
                .pread_with::<u32>(cur_line_index - 1, LE)?;
        }

        pixel_buf_index += (skip_block_bytes * start_block) * 4;
        let step = if dir & 1 != 0 { 1 } else { -1 };

        for i in start_block..block_limit {
            let mut w: i64 = width as i64 - i as i64 * TLG6_W_BLOCK_SIZE as i64;
            if w > TLG6_W_BLOCK_SIZE as i64 {
                w = TLG6_W_BLOCK_SIZE as i64;
            }
            let ww = w;
            if step == -1 {
                pixel_buf_index = (pixel_buf_index as i64 + (ww - 1) * 4) as usize;
            }
            if i & 1 != 0 {
                pixel_buf_index = (pixel_buf_index as i64 + (odd_skip * ww) * 4) as usize;
            }

            let filter_fn = if (filter_types.get(i).unwrap_or(&0) & 1) != 0 {
                avg
            } else {
                med
            };

            'pixel_loop: loop {
                let a = pixel_buf[pixel_buf_index + 3];
                let mut r = pixel_buf[pixel_buf_index + 2];
                let mut g = pixel_buf[pixel_buf_index + 1];
                let mut b = pixel_buf[pixel_buf_index];

                transform(
                    &mut r,
                    &mut g,
                    &mut b,
                    filter_types.get(i).unwrap_or(&0) >> 1,
                );
                let u = self
                    .get_prev_line()
                    .pread_with::<u32>(prev_line_index, LE)?;
                p = filter_fn(
                    p,
                    u,
                    up,
                    (0xFF0000 & ((b as u32) << 16))
                        + (0xFF00 & (g as u32) << 8)
                        + (0xFF & r as u32)
                        + ((a as u32) << 24),
                );

                if colors == 3 {
                    p |= 0xFF000000;
                }

                up = u;
                self.get_cur_line()
                    .pwrite_with::<u32>(p, cur_line_index, LE)?;

                cur_line_index += 4;
                prev_line_index += 4;
                pixel_buf_index = (pixel_buf_index as i64 + step * 4) as usize;

                w -= 1;
                if w <= 0 {
                    break 'pixel_loop;
                }
            }

            pixel_buf_index = (pixel_buf_index as i64
                + (skip_block_bytes as i64 + (if step == 1 { -ww } else { 1 })) * 4)
                as usize;
            if i & 1 != 0 {
                pixel_buf_index = (pixel_buf_index as i64 - (odd_skip * ww) * 4) as usize;
            }
        }
        self.increment_offsets();
        Ok(())
    }
}
