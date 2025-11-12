/// Returns a single set bit based on the argmuent
#[macro_export]
macro_rules! bit {
    ($x:expr) => {{
        const _: () = assert!($x <= 16, "argument must be between 0 and 16");
        1 << $x
    }};
}

/// Checks if a given `bit` is set in an integer value (`u8`, `u16`, etc.)
#[macro_export]
macro_rules! isbitset {
    ($value:expr, $bit:expr) => {{
        let bits = core::mem::size_of_val(&$value) * 8;
        assert!($bit < bits, "bit index out of range for type");
        ($value & (1 << $bit)) != 0
    }};
}

/// Sets the specified `bit` in a mutable integer variable
#[macro_export]
macro_rules! setbit {
    ($value:expr, $bit:expr) => {{
        let bits = core::mem::size_of_val(&$value) * 8;
        assert!($bit < bits, "bit index out of range for type");
        $value |= 1 << $bit;
    }};
}

/// Clears the specified `bit` in a mutable integer variable
#[macro_export]
macro_rules! clearbit {
    ($value:expr, $bit:expr) => {{
        let bits = core::mem::size_of_val(&$value) * 8;
        assert!($bit < bits, "bit index out of range for type");
        $value &= !(1 << $bit);
    }};
}

/// Toggles (flips) the specified `bit` in a mutable integer variable
#[macro_export]
macro_rules! togglebit {
    ($value:expr, $bit:expr) => {{
        let bits = core::mem::size_of_val(&$value) * 8;
        assert!($bit < bits, "bit index out of range for type");
        $value ^= 1 << $bit;
    }};
}

/// Custom assertion with failure print in hex
#[macro_export]
macro_rules! assert_hex_eq {
    ($a:expr, $b:expr) => {
        assert!($a == $b, "assertion failed: {:#06x} != {:#06x}", $a, $b);
    };
}

/// Asserts that a serial buffer contains the expected string
///
/// # Examples
///
/// ```ignore
/// assert_blargg!(gb.serial.buffmt(), "Hello");
/// ```
#[macro_export]
macro_rules! assert_blargg {
    ($buffer:expr, $expected:expr) => {
        assert_eq!(
            $buffer,
            $expected,
            "Serial buffer mismatch:\n  expected: {:?}\n  got:      {:?}",
            $expected,
            $buffer
        );
    };
}
