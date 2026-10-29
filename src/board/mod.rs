pub mod board;

#[cfg(feature = "board-esp32c3-mini-1")]
mod board_super_mini;

#[cfg(feature = "board-custom")]
mod board_custom;
