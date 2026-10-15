//! POCSAG protocol constants.

/// POCSAG sync codeword (normal polarity).
pub const POCSAG_SYNC_CODE: u32 = 0x7CD2_15D8;

/// POCSAG sync codeword with inverted polarity.
pub const POCSAG_SYNC_INVERTED: u32 = 0x832D_EA27;

/// IDLE codeword — sent between batches and after messages.
pub const POCSAG_IDLE_CODE: u32 = 0x7A89_C197;

/// Maximum number of messages in the RX ring buffer.
pub const RX_BUFFER_SIZE: usize = 4;

/// Maximum number of CAP (RIC) filters.
pub const MAX_CAP_FILTERS: usize = 16;

/// Maximum decoded text length.
pub const MAX_TEXT_LEN: usize = 256;

/// CC1101 TX power levels (PATABLE values for 433 MHz).
pub mod tx_power {
    pub const MIN:    u8 = 0x12; // -30 dBm
    pub const LOW:    u8 = 0x0E; // -20 dBm
    pub const MEDIUM: u8 = 0x27; // -10 dBm
    pub const HIGH:   u8 = 0x50; //   0 dBm
    pub const MAX:    u8 = 0xC0; // +10 dBm
}

/// Supported POCSAG baud rates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaudRate {
    B512  = 512,
    B1200 = 1200,
    B2400 = 2400,
}

impl BaudRate {
    pub const fn hz(self) -> u32 {
        self as u32
    }

    pub const fn bit_period_us(self) -> u32 {
        match self {
            BaudRate::B512  => 1953,
            BaudRate::B1200 => 833,
            BaudRate::B2400 => 417,
        }
    }
}