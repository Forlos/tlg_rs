#![allow(clippy::many_single_char_names)]
use lazy_static::lazy_static;
use scroll::{self, Pread, LE};

const TLG6_GOLOMB_N_COUNT: usize = 4;

const LEADING_ZERO_TABLE_BITS: u8 = 12;
const LEADING_ZERO_TABLE_SIZE: usize = 1 << LEADING_ZERO_TABLE_BITS;

lazy_static! {
    static ref LEADING_ZERO_TABLE: [u8; LEADING_ZERO_TABLE_SIZE] = {
        let mut table = [0u8; LEADING_ZERO_TABLE_SIZE];
        for (i, item) in table.iter_mut().enumerate() {
            let mut count = 0;
            let mut j = 1;
            while j != LEADING_ZERO_TABLE_SIZE && (i & j) == 0 {
                j <<= 1;
                count += 1;
            }
            count += 1;
            if j == LEADING_ZERO_TABLE_SIZE {
                count = 0;
            }
            *item = count;
        }
        table
    };
    static ref GOLOMB_BIT_LENGTH_TABLE: [[u8; TLG6_GOLOMB_N_COUNT]; TLG6_GOLOMB_N_COUNT * 2 * 128] = {
        let mut table = [[0u8; TLG6_GOLOMB_N_COUNT]; TLG6_GOLOMB_N_COUNT * 2 * 128];
        let golomb_compression_table = vec![
            [3, 7, 15, 27, 63, 108, 223, 448, 130],
            [3, 5, 13, 24, 51, 95, 192, 384, 257],
            [2, 5, 12, 21, 39, 86, 155, 320, 384],
            [2, 3, 9, 18, 33, 61, 129, 258, 511],
        ];
        for (n, row) in golomb_compression_table.iter().enumerate() {
            let mut a = 0;
            for (i, item) in row.iter().enumerate() {
                for _ in 0..*item {
                    table[a][n] = i as u8;
                    a += 1;
                }
            }
        }
        table
    };
}

pub(crate) fn decode_golomb(
    input_buf: &mut [u8],
    pixel_count: usize,
    bit_pool: &[u8],
) -> anyhow::Result<()> {
    let mut n: i32 = TLG6_GOLOMB_N_COUNT as i32 - 1;
    let mut a: i32 = 0;
    let mut bit_pos: u32 = 1;
    let mut zero = (bit_pool[0] & 1) == 0;

    let mut input_buf_index: usize = 0;
    let mut bit_pool_index: usize = 0;

    while input_buf_index < pixel_count * 4 {
        let mut t = bit_pool.pread_with::<u32>(bit_pool_index, LE)? >> bit_pos;
        let mut b = LEADING_ZERO_TABLE[t as usize & (LEADING_ZERO_TABLE_SIZE - 1)];
        let mut bit_count = b as u32;
        while b == 0 {
            bit_count += LEADING_ZERO_TABLE_BITS as u32;
            bit_pos += LEADING_ZERO_TABLE_BITS as u32;
            bit_pool_index += bit_pos as usize >> 3;
            bit_pos &= 7;
            t = bit_pool.pread_with::<u32>(bit_pool_index, LE)? >> bit_pos;
            b = LEADING_ZERO_TABLE[t as usize & (LEADING_ZERO_TABLE_SIZE - 1)];
            bit_count += b as u32;
        }
        bit_pos += b as u32;
        bit_pool_index += bit_pos as usize >> 3;
        bit_pos &= 7;
        bit_count -= 1;
        let mut count = 1 << bit_count;
        t = bit_pool.pread_with::<u32>(bit_pool_index, LE)?;
        count += (t >> bit_pos) & (count - 1);
        bit_pos += bit_count;
        bit_pool_index += bit_pos as usize >> 3;
        bit_pos &= 7;

        if zero {
            'zero_loop: loop {
                input_buf[input_buf_index] = 0;
                input_buf_index += 4;

                count -= 1;
                if count == 0 {
                    break 'zero_loop;
                }
            }
        } else {
            'non_zero_loop: loop {
                t = bit_pool.pread_with::<u32>(bit_pool_index, LE)? >> bit_pos;
                if t != 0 {
                    b = LEADING_ZERO_TABLE[t as usize & (LEADING_ZERO_TABLE_SIZE - 1)];
                    bit_count = b as u32;
                    while b == 0 {
                        bit_count += LEADING_ZERO_TABLE_BITS as u32;
                        bit_pos += LEADING_ZERO_TABLE_BITS as u32;
                        bit_pool_index += bit_pos as usize >> 3;
                        bit_pos &= 7;
                        t = bit_pool.pread_with::<u32>(bit_pool_index, LE)? >> bit_pos;
                        b = LEADING_ZERO_TABLE[t as usize & (LEADING_ZERO_TABLE_SIZE - 1)];
                        bit_count += b as u32;
                    }
                    bit_count -= 1;
                } else {
                    bit_pool_index += 5;
                    bit_count = bit_pool[bit_pool_index - 1] as u32;
                    bit_pos = 0;
                    t = bit_pool.pread_with::<u32>(bit_pool_index, LE)?;
                    b = 0;
                }
                let k = GOLOMB_BIT_LENGTH_TABLE[a as usize][n as usize];
                let mut v: i32 = (bit_count << k) as i32 + ((t >> b) as i32 & ((1 << k) - 1));
                let sign: i32 = (v & 1) - 1;
                v >>= 1;
                a += v;
                input_buf[input_buf_index] = ((v ^ sign) + sign + 1) as u8;
                input_buf_index += 4;

                bit_pos += b as u32 + k as u32;
                bit_pool_index += bit_pos as usize >> 3;
                bit_pos &= 7;

                n -= 1;
                if n < 0 {
                    a >>= 1;
                    n = TLG6_GOLOMB_N_COUNT as i32 - 1;
                }

                count -= 1;
                if count == 0 {
                    break 'non_zero_loop;
                }
            }
        }
        zero = !zero;
    }
    Ok(())
}
