use crate::pocsag::consts::{BaudRate, POCSAG_IDLE_CODE, POCSAG_SYNC_CODE};

/// Computes the BCH(31,21) + parity codeword.
///
/// Mirrors `OpenPager::bchEncode` from the original C++ library.
pub fn bch_encode(data: u32) -> u32 {
    let mut cw_e = data & 0xFFFF_F800;
    for _ in 0..21 {
        if cw_e & 0x8000_0000 != 0 {
            cw_e ^= 0xED20_0000;
        }
        cw_e <<= 1;
    }
    let mut cw = (data & 0xFFFF_F800) | (cw_e >> 21);
    // Even parity
    let mut parity = 0u32;
    for i in 0..32 {
        if (cw >> i) & 1 != 0 {
            parity ^= 1;
        }
    }
    if parity != 0 {
        cw |= 1;
    }
    cw
}

/// Creates an address codeword for a given RIC and function.
pub fn create_address_cw(ric: u32, func: u8) -> u32 {
    let raw = ((ric >> 3) << 13) | (((func & 0x03) as u32) << 11);
    bch_encode(raw)
}

/// Reverses the lower `n` bits of `x` (bit-order reversal).
fn reverse_n(x: u8, n: u8) -> u8 {
    let mut r = 0u8;
    for i in 0..8 {
        r = (r << 1) | ((x >> i) & 1);
    }
    r >> (8 - n)
}

/// Encodes an alphanumeric message into POCSAG codewords.
///
/// Returns the number of codewords written into `codewords`.
pub fn encode_alpha(msg: &[u8], codewords: &mut [u32]) -> usize {
    let mut buf: u64 = 0;
    let mut bits: u8 = 0;
    let mut idx: usize = 0;

    for &byte in msg {
        buf = (buf << 7) | reverse_n(byte, 7) as u64;
        bits += 7;
        while bits >= 20 {
            bits -= 20;
            codewords[idx] = bch_encode((1u32 << 31) | ((((buf >> bits) & 0xFFFFF) as u32) << 11));
            idx += 1;
        }
    }

    // Append EOT (0x04) and pad to 20 bits
    buf = (buf << 7) | reverse_n(0x04, 7) as u64;
    bits += 7;
    if bits > 0 {
        while bits < 20 {
            buf <<= 1;
            bits += 1;
        }
        codewords[idx] =
            bch_encode((1u32 << 31) | ((((buf >> (bits - 20)) & 0xFFFFF) as u32) << 11));
        idx += 1;
    }

    idx
}

/// Encodes a numeric (BCD) message into POCSAG codewords.
pub fn encode_numeric(msg: &[u8], codewords: &mut [u32]) -> usize {
    // Each codeword carries 5 BCD digits (20 bits)
    let mut idx = 0usize;
    let mut data: u32 = 0;
    let mut digits_in_cw: u8 = 0;

    for &bcd in msg {
        data = (data << 4) | (bcd as u32 & 0x0F);
        digits_in_cw += 1;

        if digits_in_cw == 5 {
            codewords[idx] = bch_encode((1u32 << 31) | (data << 11));
            idx += 1;
            data = 0;
            digits_in_cw = 0;
        }
    }

    // Flush remaining digits, padded with spaces (0x0C)
    if digits_in_cw > 0 {
        while digits_in_cw < 5 {
            data = (data << 4) | 0x0C;
            digits_in_cw += 1;
        }
        codewords[idx] = bch_encode((1u32 << 31) | (data << 11));
        idx += 1;
    }

    idx
}

/// Builds the full POCSAG bitstream (preamble + batches) for transmission.
///
/// Returns the total number of codewords written, or `0` on error.
///
/// The bitstream is written as a sequence of `u32` codewords. Each codeword
/// is 32 bits and must be sent MSB-first at the configured baud rate.
pub fn build_transmit_buffer(
    ric: u32,
    func: u8,
    alpha: bool,
    msg: &[u8],
    baud: BaudRate,
    out: &mut [u32],
) -> usize {
    let addr_cw = create_address_cw(ric, func);

    let mut msg_cws = [0u32; 128];
    let msg_count = if alpha {
        encode_alpha(msg, &mut msg_cws)
    } else {
        encode_numeric(msg, &mut msg_cws)
    };

    let frame = (ric & 0x07) as usize;
    let start_slot = frame * 2;
    let total_cw = 1 + msg_count;
    let batches = (start_slot + total_cw + 15).div_ceil(16);

    let mut out_idx = 0usize;

    // 20 codewords of preamble (0xAAAAAAAA)
    for _ in 0..20 {
        if out_idx >= out.len() {
            return 0;
        }
        out[out_idx] = 0xAAAA_AAAA;
        out_idx += 1;
    }

    let mut msg_idx = 0usize;
    let mut addr_sent = false;

    for _b in 0..batches {
        if out_idx >= out.len() {
            return 0;
        }
        out[out_idx] = POCSAG_SYNC_CODE;
        out_idx += 1;

        for f in 0..8 {
            for _s in 0..2 {
                if out_idx >= out.len() {
                    return 0;
                }

                if !addr_sent && f == frame {
                    out[out_idx] = addr_cw;
                    out_idx += 1;
                    addr_sent = true;
                } else if addr_sent && msg_idx < msg_count {
                    out[out_idx] = msg_cws[msg_idx];
                    out_idx += 1;
                    msg_idx += 1;
                } else {
                    out[out_idx] = POCSAG_IDLE_CODE;
                    out_idx += 1;
                }
            }
        }

        if msg_idx >= msg_count {
            break;
        }
    }

    // Trailing IDLE codewords
    for _ in 0..4 {
        if out_idx >= out.len() {
            return 0;
        }
        out[out_idx] = POCSAG_IDLE_CODE;
        out_idx += 1;
    }

    out_idx
}
