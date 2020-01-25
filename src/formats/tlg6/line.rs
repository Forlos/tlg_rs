use scroll::{self, ctx, Pread, LE};

#[derive(Debug)]
pub(crate) struct Tlg6Line {
    pub(crate) bits: Vec<Tlg6Bits>,
}

impl<'a> ctx::TryFromCtx<'a, usize> for Tlg6Line {
    type Error = failure::Error;
    #[inline]
    fn try_from_ctx(this: &'a [u8], size: usize) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let mut bits = Vec::with_capacity(size);
        for _ in 0..size {
            bits.push(this.gread_with(offset, LE)?);
        }
        Ok((Tlg6Line { bits }, *offset as usize))
    }
}

#[derive(Debug)]
pub(crate) struct Tlg6Bits {
    size: u32,
    pub(crate) bit_pool: Vec<u8>,
    pub(crate) method: u32,
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for Tlg6Bits {
    type Error = failure::Error;
    #[inline]
    fn try_from_ctx(this: &'a [u8], _: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let size = this.pread_with::<u32>(0, LE)?;
        let length = if size % 8 == 0 {
            size / 8
        } else {
            size / 8 + 1
        };
        let mut bit_pool = this[4..length as usize + 4].to_vec();
        // TODO this looks like big workaround check if it is really needed.
        bit_pool.extend(vec![0; 5]);
        let method = (size >> 30) & 3;
        Ok((
            Tlg6Bits {
                size,
                bit_pool,
                method,
            },
            length as usize + 4,
        ))
    }
}
