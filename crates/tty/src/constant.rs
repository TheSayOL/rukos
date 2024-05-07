pub const LF: u8 = b'\n';
pub const CR: u8 = b'\r';

pub const DEL: u8 = b'\x7f';
pub const BS: u8 = b'\x08';

pub const SPACE: u8 = b' ';

/// escape
pub const ESC: u8 = 27;
/// [
pub const LEFT_BRACKET: u8 = 91;

/// an arrow char is `ARROW_PREFIX` + `UP/DOWN/RIGHT/LEFT`
pub const ARROW_PREFIX: [u8; 2] = [ESC, LEFT_BRACKET];

// const UP: u8 = 65;
// const DOWN: u8 = 66;
pub const RIGHT: u8 = 67;
pub const LEFT: u8 = 68;
