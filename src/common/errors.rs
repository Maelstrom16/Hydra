use std::fmt;

#[derive(Debug)]
pub enum HydraIOError {
    InvalidEmulator(&'static str, Option<String>),
    InvalidInstruction(u64, usize),
    MalformedROM(&'static str),
    OpenBusAccess,

    IOError(std::io::Error),
    DeserializationError(toml::de::Error),
    SerializationError(toml::ser::Error),
}

impl fmt::Display for HydraIOError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HydraIOError::InvalidEmulator(emulator, extension) => match extension {
                Some(ext) => write!(f, "{} does not support ROM files with a .{} extension", emulator, ext),
                None => write!(f, "{} does not support extensionless ROM files", emulator),
            },
            HydraIOError::InvalidInstruction(value, address) => write!(f, "Attempted to execute invalid instruction {} at address {}", value, address),
            HydraIOError::MalformedROM(details) => write!(f, "Malformed ROM file: {}", details),
            HydraIOError::OpenBusAccess => {
                write!(f, "Attempted to read from an unmapped memory block")
            }

            HydraIOError::IOError(error) => write!(f, "{}", error),
            HydraIOError::DeserializationError(error) => write!(f, "{}", error),
            HydraIOError::SerializationError(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for HydraIOError {}

impl From<std::io::Error> for HydraIOError {
    fn from(err: std::io::Error) -> Self {
        HydraIOError::IOError(err)
    }
}

impl From<toml::de::Error> for HydraIOError {
    fn from(err: toml::de::Error) -> Self {
        HydraIOError::DeserializationError(err)
    }
}

impl From<toml::ser::Error> for HydraIOError {
    fn from(err: toml::ser::Error) -> Self {
        HydraIOError::SerializationError(err)
    }
}

#[macro_export]
macro_rules! propagate {
    ($expr:expr) => {
        (|| -> Result<_, HydraIOError> { $expr })()
    };
}
#[macro_export]
macro_rules! propagate_or {
    ($expr:expr, $def:expr) => {
        propagate!($expr).unwrap_or($def)
    };
}
#[macro_export]
macro_rules! propagate_or_else {
    ($expr:expr, $func:expr) => {
        propagate!($expr).unwrap_or_else($func)
    };
}
