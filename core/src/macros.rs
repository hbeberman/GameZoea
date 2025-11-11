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
