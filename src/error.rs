use std;
use std::convert::From;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Message(String),
    UnknownSegmentType,
    LengthNotFound,
    StackProblem,
    NonUtf8Str,
    UnsupportedType,
    Eof,
    ParsingLength,
    UnusedParseData,
    ParsingUnit,
    ParsingBool,
    ParsingMap,
    ParsingEnum,
    ParsingUnsigned,
    ParsingString,
    ParsingSeq,
    ParsingUnitVariant,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(error: std::string::FromUtf8Error) -> Self {
        Error::NonUtf8Str
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(std::error::Error::description(self))
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Message(ref msg) => msg,
            Error::UnknownSegmentType => "unknown segment type",
            Error::LengthNotFound => "length not found but required",
            Error::StackProblem => "stack problem",
            Error::UnusedParseData => "unused parse data",
            Error::ParsingUnit => "error parsing unit",
            Error::ParsingBool => "error parsing bool",
            Error::ParsingMap => "error parsing map",
            Error::ParsingEnum => "error parsing enum",
            Error::ParsingUnsigned => "error parsing unsigned",
            Error::ParsingString => "error parsing string",
            Error::ParsingSeq => "error parsing sequence",
            Error::ParsingUnitVariant => "error parsing unit variant",
            Error::Eof => "error eof",
            Error::UnsupportedType => "unsupported type",
            Error::ParsingLength => "error parsing data length",
            Error::NonUtf8Str => "error parsing string that wasn't utf8",
        }
    }
}
