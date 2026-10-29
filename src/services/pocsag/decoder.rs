use crate::services::pocsag::consts::{BaudRate, POCSAG_IDLE_CODE, POCSAG_SYNC_CODE, POCSAG_SYNC_INVERTED};
use crate::services::pocsag::encoder::bch_encode;
use crate::services::pocsag::message::OpenPagerMessage;

/// Software POCSAG decoder for a single baud rate.
///
/// Feed raw bits (sampled from GDO0) into [`process_sample`](Self::process_sample).
/// When a complete message is decoded, the callback is invoked.
pub struct PocsagDecoder<F>
where
    F: FnMut(OpenPagerMessage),
{
    baud: BaudRate,
    bit_period_us: u32,
    callback: F,

    // Inversion
    invert: bool,
    inverted: bool,

    // Sync state
    shift_reg: u32,
    bit_count: u8,
    synced: bool,

    // Batch state
    codewords: [u32; 16],
    cw_index: usize,

    // Message assembly
    current_ric: u32,
    current_func: u8,
    msg_cws: [u32; 64],
    msg_cw_count: usize,
    in_message: bool,

    // Timing (software PLL)
    last_bit: bool,
    next_sample_time: u32,
    last_msg_activity_us: u32,
}

impl<F> PocsagDecoder<F>
where
    F: FnMut(OpenPagerMessage),
{
    pub fn new(baud: BaudRate, callback: F) -> Self {
        Self {
            baud,
            bit_period_us: baud.bit_period_us(),
            callback,
            invert: false,
            inverted: false,
            shift_reg: 0,
            bit_count: 0,
            synced: false,
            codewords: [0u32; 16],
            cw_index: 0,
            current_ric: 0,
            current_func: 0,
            msg_cws: [0u32; 64],
            msg_cw_count: 0,
            in_message: false,
            last_bit: false,
            next_sample_time: 0,
            last_msg_activity_us: 0,
        }
    }

    pub fn set_invert(&mut self, invert: bool) {
        self.invert = invert;
    }

    /// Feed a single bit sample at the current timestamp (microseconds).
    ///
    /// This implements the same software-PLL edge-detection logic as the
    /// original `OpenPagerDecoder::process()`.
    pub fn process_sample(&mut self, now: u32, bit: bool) {
        let bit = if self.invert { !bit } else { bit };

        // Edge detection — resync to middle of bit period
        if bit != self.last_bit {
            self.last_bit = bit;
            self.next_sample_time = now.wrapping_add(self.bit_period_us / 2);
        }

        // Sample at the centre of each bit period
        if self.next_sample_time != 0 && (now.wrapping_sub(self.next_sample_time) as i32) >= 0 {
            self.sample_bit(bit);
            self.next_sample_time = self.next_sample_time.wrapping_add(self.bit_period_us);

            // Anti-runaway
            if (now.wrapping_sub(self.next_sample_time) as i32) > (self.bit_period_us * 3) as i32 {
                self.next_sample_time = 0;
            }
        }

        // Message timeout
        if self.in_message && self.msg_cw_count > 0 {
            let timeout = self.bit_period_us * 32 * 20;
            if (now.wrapping_sub(self.last_msg_activity_us) as i32) > timeout as i32 {
                self.finalize_message();
            }
        }
    }

    /// Feed a chunk of raw bit samples (e.g. from a DMA buffer).
    ///
    /// Each entry is `(timestamp_us, bit)`.
    pub fn process_batch2(&mut self, samples: &[(u32, bool)]) {
        for &(ts, bit) in samples {
            self.process_sample(ts, bit);
        }
    }

    fn sample_bit(&mut self, bit: bool) {
        self.shift_reg = (self.shift_reg << 1) | (bit as u32);
        self.bit_count += 1;

        if !self.synced {
            if self.check_sync_word() {
                self.synced = true;
                self.bit_count = 0;
                self.cw_index = 0;
            }
        } else if self.bit_count == 32 {
            self.codewords[self.cw_index] = self.shift_reg;
            self.cw_index += 1;
            self.bit_count = 0;

            if self.cw_index >= 16 {
                self.process_batch();
                self.synced = false;
                self.cw_index = 0;
            }
        }
    }

    fn check_sync_word(&self) -> bool {
        if bit_diff(self.shift_reg, POCSAG_SYNC_CODE) <= 2 {
            // Normal polarity — note: we cannot mutate self here,
            // so the caller must handle inversion detection.
            true
        } else if bit_diff(self.shift_reg, POCSAG_SYNC_INVERTED) <= 2 {
            true
        } else {
            false
        }
    }

    fn process_batch(&mut self) {
        for i in 0..16 {
            let mut cw = self.codewords[i];

            // Apply inversion if sync was inverted
            if self.inverted {
                cw = !cw;
            }

            if cw == POCSAG_IDLE_CODE {
                if self.in_message {
                    self.finalize_message();
                }
                continue;
            }

            // BCH check & repair
            let bch_ok = bch_check(cw);
            if !bch_ok {
                let corrected = bch_repair(cw);
                if corrected != 0 {
                    cw = corrected;
                } else {
                    continue;
                }
            }

            let is_message = (cw >> 31) & 1 != 0;

            if !is_message {
                // Address codeword
                if self.in_message {
                    self.finalize_message();
                }
                let address = (cw >> 13) & 0x7ffff;
                let func = ((cw >> 11) & 0x03) as u8;
                self.current_ric = (address << 3) | ((i as u32) / 2);
                self.current_func = func;
                self.in_message = true;
                self.msg_cw_count = 0;
            } else if self.in_message {
                if self.msg_cw_count < 64 {
                    self.msg_cws[self.msg_cw_count] = cw;
                    self.msg_cw_count += 1;
                }
            }
        }
        self.last_msg_activity_us = 0; // caller should update with real time
    }

    fn finalize_message(&mut self) {
        let mut msg = OpenPagerMessage::new();
        msg.ric = self.current_ric;
        msg.func = self.current_func;
        msg.baud_rate = self.baud.hz();
        msg.is_numeric = self.current_func == 0 || self.current_func == 1;
        msg.valid = true;

        if self.msg_cw_count == 0 {
            // Tone-only
            let tone = b"[Tone Only]";
            msg.text[..tone.len()].copy_from_slice(tone);
            msg.text_len = tone.len();
        } else if msg.is_numeric {
            decode_numeric(&self.msg_cws[..self.msg_cw_count], &mut msg);
        } else {
            decode_alpha(&self.msg_cws[..self.msg_cw_count], &mut msg);
        }

        (self.callback)(msg);

        self.in_message = false;
        self.msg_cw_count = 0;
    }
}

// ── BCH helpers ──────────────────────────────────────────────────────

fn bit_diff(a: u32, b: u32) -> u32 {
    (a ^ b).count_ones()
}

fn bch_check(cw: u32) -> bool {
    bch_encode(cw) == cw
}

fn bch_repair(cw: u32) -> u32 {
    if bch_check(cw) {
        return cw;
    }
    // Try single-bit correction
    for i in 0..32 {
        let corrected = cw ^ (1u32 << i);
        if bch_check(corrected) {
            return corrected;
        }
    }
    0
}

// ── Message decoders ─────────────────────────────────────────────────

fn decode_alpha(cws: &[u32], msg: &mut OpenPagerMessage) {
    let mut bit_buf: u64 = 0;
    let mut bit_count: u8 = 0;
    let mut text_idx = 0usize;

    for &cw in cws {
        let data = (cw >> 11) & 0xfffff;
        bit_buf = (bit_buf << 20) | data as u64;
        bit_count += 20;

        while bit_count >= 7 && text_idx < 254 {
            bit_count -= 7;
            let ch = ((bit_buf >> bit_count) & 0x7f) as u8;

            // Reverse 7 bits
            let ch = ch.reverse_bits() >> 1;

            if ch == 0x04 || ch == 0x00 {
                msg.text[text_idx] = 0;
                msg.text_len = text_idx;
                return;
            }

            msg.text[text_idx] = ch;
            text_idx += 1;
        }
    }

    msg.text[text_idx] = 0;
    msg.text_len = text_idx;
}

fn decode_numeric(cws: &[u32], msg: &mut OpenPagerMessage) {
    let mut text_idx = 0usize;

    for &cw in cws {
        let data = (cw >> 11) & 0xfffff;
        for d in (0..5).rev() {
            if text_idx >= 254 {
                break;
            }
            let bcd = ((data >> (d * 4)) & 0x0f) as u8;
            msg.text[text_idx] = bcd_to_char(bcd) as u8;
            text_idx += 1;
        }
    }

    msg.text[text_idx] = 0;
    msg.text_len = text_idx;
}

fn bcd_to_char(bcd: u8) -> char {
    // Reverse 4 bits
    let bcd = bcd.reverse_bits() >> 4;
    match bcd {
        0..=9 => (b'0' + bcd) as char,
        0x0a => '*',
        0x0b => 'U',
        0x0c => ' ',
        0x0d => '-',
        0x0e => ')',
        0x0f => '(',
        _ => '?',
    }
}
