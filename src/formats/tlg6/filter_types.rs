use scroll::{self, ctx, Pread, LE};

lazy_static! {
    static ref STATE: Vec<u8> = {
        let mut v = vec![0; 4096];
        let mut index = 0;
        for i in 0..32 {
            for j in 0..16 {
                for _ in 0..4 {
                    v[index] = i;
                    index += 1;
                }
                for _ in 0..4 {
                    v[index] = j;
                    index += 1;
                }
            }
        }
        v
    };
}

#[derive(Debug)]
pub(crate) struct Tlg6FilterTypes {
    size: u32,
    buf: Vec<u8>,
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for Tlg6FilterTypes {
    type Error = failure::Error;
    #[inline]
    fn try_from_ctx(this: &'a [u8], _: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let size = this.pread_with::<u32>(0, LE)?;
        let buf = this[4..size as usize + 4].to_vec();
        Ok((Tlg6FilterTypes { size, buf }, size as usize + 4))
    }
}

impl Tlg6FilterTypes {
    pub(crate) fn decompress(&self, output_size: usize) -> Vec<u8> {
        let mut internal_state = STATE.clone();
        let mut output: Vec<u8> = Vec::with_capacity(output_size);
        let mut index = 0usize;
        let mut state_index = 0usize;
        let mut flags = 0u32;

        while index < self.size as usize {
            flags >>= 1;
            if (flags & 0x100) != 0x100 {
                flags = self.buf[index] as u32 | 0xFF00;
                index += 1
            }
            if (flags & 1) == 1 {
                let x0 = self.buf[index];
                index += 1;
                let x1 = self.buf[index];
                index += 1;
                let mut position: u32 = (x0 as u32) | (((x1 & 0xF) as u32) << 8);
                let mut length: usize = 3 + ((x1 as usize & 0xF0) >> 4);
                if length == 18 {
                    length += self.buf[index] as usize;
                    index += 1;
                }
                for _ in 0..length {
                    let c = internal_state[position as usize];
                    output.push(c);
                    internal_state[state_index] = c;
                    state_index += 1;
                    state_index &= 0xFFF;
                    position += 1;
                    position &= 0xFFF;
                }
            } else {
                let c = self.buf[index];
                index += 1;
                output.push(c);
                internal_state[state_index] = c;
                state_index += 1;
                state_index &= 0xFFF;
            }
        }
        // Initialize rest of vector with zeros
        output.extend(vec![0; output_size - output.len()]);

        output
    }
}
