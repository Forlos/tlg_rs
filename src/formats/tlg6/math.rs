#![allow(clippy::many_single_char_names)]
#[inline]
pub(crate) fn avg(a: u32, b: u32, _: u32, v: u32) -> u32 {
    packed_bytes_add(
        (a & b) + (((a ^ b) & 0xFEFE_FEFE) >> 1) + ((a ^ b) & 0x0101_0101),
        v,
    )
}

#[inline]
pub(crate) fn med(a: u32, b: u32, c: u32, v: u32) -> u32 {
    let mask = gt_mask(a, b);
    let ab = (a ^ b) & mask;
    let aa = ab ^ a;
    let bb = ab ^ b;
    let a_mask = gt_mask(aa, c);
    let b_mask = gt_mask(c, bb);
    let m = !(a_mask | b_mask);
    packed_bytes_add(
        (b_mask & aa) | (a_mask & bb) | ((bb & m) - (c & m) + (aa & m)),
        v,
    )
}

#[inline]
fn packed_bytes_add(a: u32, b: u32) -> u32 {
    a.wrapping_add(b)
        .wrapping_sub(((a & b) << 1).wrapping_add((a ^ b) & 0xFEFE_FEFE) & 0x0101_0100)
}

#[inline]
fn gt_mask(a: u32, b: u32) -> u32 {
    let x = !b;
    let y = ((a & x) + (((a ^ x) >> 1) & 0x7F7F_7F7F)) & 0x8080_8080;
    (y >> 7).wrapping_add(0x7F7F_7F7F) ^ 0x7F7F_7F7F
}
