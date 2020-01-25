pub const TLG6_W_BLOCK_SIZE: usize = 8;
pub const TLG6_H_BLOCK_SIZE: usize = 8;

pub const TLG_MAGIC_SIZE: usize = 11;

/// TLG6.0\x00raw\x1a
pub const TLG6_MAGIC: [u8; TLG_MAGIC_SIZE] = [84, 76, 71, 54, 46, 48, 0, 114, 97, 119, 26];
