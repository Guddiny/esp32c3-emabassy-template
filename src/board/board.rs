#[cfg(not(any(feature = "board-esp32c3-mini-1", feature = "board-custom")))]
compile_error!("no board selected: enable `board-esp32c3-mini-1` or `board-custom`");

#[cfg(all(feature = "board-esp32c3-mini-1", feature = "board-custom"))]
compile_error!("multiple boards selected: enable exactly one variant feature");

#[cfg(feature = "board-esp32c3-mini-1")]
pub use super::board_super_mini::*;
#[cfg(feature = "board-custom")]
pub use super::board_custom::*;
