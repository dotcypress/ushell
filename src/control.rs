/// Null
pub const NULL: u8 = 0x0;

/// Start of heading: CTRL-A
pub const SOH: u8 = 0x1;
pub const CTRL_A: u8 = SOH;

/// Start of text: CTRL-B
pub const STX: u8 = 0x2;
pub const CTRL_B: u8 = STX;

/// End of text: CTRL-C
pub const ETX: u8 = 0x3;
pub const CTRL_C: u8 = ETX;

/// End of transmission: CTRL-D
pub const EOT: u8 = 0x4;
pub const CTRL_D: u8 = EOT;

/// Enquiry: CTRL-E
pub const ENQ: u8 = 0x5;
pub const CTRL_E: u8 = ENQ;

/// Acknowledge: CTRL-F
pub const ACK: u8 = 0x6;
pub const CTRL_F: u8 = ACK;

/// Bell: CTRL-G
pub const BELL: u8 = 0x7;
pub const CTRL_G: u8 = BELL;

/// Backspace: CTRL-H
pub const BS: u8 = 0x8;
pub const CTRL_H: u8 = BS;

/// Character tabulation: CTRL-I
pub const TAB: u8 = 0x9;
pub const CTRL_I: u8 = TAB;

/// Line feed: CTRL-J
pub const LF: u8 = 0xa;
pub const CTRL_J: u8 = LF;

/// Line tabulation: CTRL-K
pub const VT: u8 = 0xb;
pub const CTRL_K: u8 = VT;

/// Form feed: CTRL-L
pub const FF: u8 = 0xc;
pub const CTRL_L: u8 = FF;

/// Carriage return: CTRL-M
pub const CR: u8 = 0xd;
pub const CTRL_M: u8 = CR;

/// Shift out: CTRL-N
pub const SO: u8 = 0xe;
pub const CTRL_N: u8 = SO;

/// Shift in: CTRL-O
pub const SI: u8 = 0xf;
pub const CTRL_O: u8 = SI;

/// Data link escape: CTRL-P
pub const DLE: u8 = 0x10;
pub const CTRL_P: u8 = DLE;

/// Device control one: CTRL-Q
pub const DC1: u8 = 0x11;
pub const CTRL_Q: u8 = DC1;

/// Device control two: CTRL-R
pub const DC2: u8 = 0x12;
pub const CTRL_R: u8 = DC2;

/// Device control three: CTRL-S
pub const DC3: u8 = 0x13;
pub const CTRL_S: u8 = DC3;

/// Device control four: CTRL-T
pub const DC4: u8 = 0x14;
pub const CTRL_T: u8 = DC4;

/// Negative acknowledge: CTRL-U
pub const NAK: u8 = 0x15;
pub const CTRL_U: u8 = NAK;

/// Synchronous idle: CTRL-V
pub const SYN: u8 = 0x16;
pub const CTRL_V: u8 = SYN;

/// End of transmission block: CTRL-W
pub const ETB: u8 = 0x17;
pub const CTRL_W: u8 = ETB;

/// Cancel: CTRL-X
pub const CAN: u8 = 0x18;
pub const CTRL_X: u8 = CAN;

/// End of medium: CTRL-Y
pub const EM: u8 = 0x19;
pub const CTRL_Y: u8 = EM;

/// Substitute: CTRL-Z
pub const SUB: u8 = 0x1a;
pub const CTRL_Z: u8 = SUB;

/// Escape: CTRL-[
pub const ESC: u8 = 0x1b;
pub const CTRL_LEFT_BRACKET: u8 = ESC;

/// Information separator four: CTRL-\
pub const FS: u8 = 0x1c;
pub const CTRL_BACKSLASH: u8 = FS;

/// Information separator three: CTRL-]
pub const GS: u8 = 0x1d;
pub const CTRL_RIGHT_BRACKET: u8 = GS;

/// Information separator two: CTRL-^
pub const RS: u8 = 0x1e;
pub const CTRL_CARET: u8 = RS;

/// Information separator one: CTRL-_
pub const US: u8 = 0x1f;
pub const CTRL_UNDERLINE: u8 = US;

///Delete: delete
pub const DEL: u8 = 0x7f;
