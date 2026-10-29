use crate::services::pocsag::consts::MAX_TEXT_LEN;

/// A decoded POCSAG message.
#[derive(Clone)]
pub struct OpenPagerMessage {
    /// Receiver ID Code (capcode).
    pub ric: u32,
    /// Function bits (0–3).
    pub func: u8,
    /// Decoded text (null-padded).
    pub text: [u8; MAX_TEXT_LEN],
    /// Actual text length.
    pub text_len: usize,
    /// `true` if numeric (BCD) encoding.
    pub is_numeric: bool,
    /// `true` if BCH check passed.
    pub valid: bool,
    /// Detected baud rate.
    pub baud_rate: u32,
}

impl OpenPagerMessage {
    pub fn new() -> Self {
        Self {
            ric: 0,
            func: 0,
            text: [0u8; MAX_TEXT_LEN],
            text_len: 0,
            is_numeric: false,
            valid: false,
            baud_rate: 1200,
        }
    }

    /// Returns the decoded text as a `&str` slice.
    pub fn text_str(&self) -> &str {
        core::str::from_utf8(&self.text[..self.text_len]).unwrap_or("[invalid UTF-8]")
    }
}

impl Default for OpenPagerMessage {
    fn default() -> Self {
        Self::new()
    }
}
