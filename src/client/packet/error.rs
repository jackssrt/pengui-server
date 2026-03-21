use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum PacketError {
    Message(String),
    Incomplete,
    Invalid(&'static str),
}
impl std::error::Error for PacketError {}
impl Display for PacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(msg) => f.write_str(msg),
            Self::Incomplete => f.write_str("incomplete data"),
            Self::Invalid(what) => write!(f, "invalid {what}"),
        }
    }
}
impl serde::ser::Error for PacketError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Message(msg.to_string())
    }
}

impl serde::de::Error for PacketError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Message(msg.to_string())
    }
}
