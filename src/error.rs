use std::error::Error;
use std::string::FromUtf8Error;
use std::fmt;
use std::io::Error as IOError;
use serde_json::error::Error as JsonError;
use ureq::Error as RequestError;

#[derive(Debug)]
pub enum ArchiveError {
    Compression(IOError),
    Archive(IOError),
    FormatSupport(String),
}

impl Error for ArchiveError {}

impl fmt::Display for ArchiveError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            ArchiveError::Compression(msg) => {
                write!(formatter, "Decompression failed: {}", msg)
            },
            ArchiveError::Archive(e) => {
                write!(formatter, "Invalid archive: {}", e)
            },
            ArchiveError::FormatSupport(e) => {
                write!(formatter, "Format not supported: {}", e)
            },
        };
    }
}

#[derive(Debug)]
pub enum InvalidInput {
    Archive(ArchiveError),
    Utf8(FromUtf8Error),
    Json(JsonError),
    MissingData(String),
    FormatSupport(String),
}

impl Error for InvalidInput {}

impl fmt::Display for InvalidInput {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            InvalidInput::Archive(err) => {
                write!(formatter, "Invalid tar: {}", err)
            },
            InvalidInput::Utf8(err) =>{
                write!(formatter, "Invalid Utf8: {}", err)
            },
            InvalidInput::Json(err) => {
                write!(formatter, "Invalid json: {}", err)
            },
            InvalidInput::MissingData(err) => {
                write!(formatter, "Missing Data: {}", err)
            }
            InvalidInput::FormatSupport(err) => {
                write!(formatter, "Missing Data: {}", err)
            }
        };
    }
}

impl From<ArchiveError> for InvalidInput {
    fn from(err: ArchiveError) -> Self {
        return InvalidInput::Archive(err);
    }
}

impl From<FromUtf8Error> for InvalidInput {
    fn from(err: FromUtf8Error) -> Self {
        return InvalidInput::Utf8(err);
    }
}

impl From<JsonError> for InvalidInput {
    fn from(err: JsonError) -> Self {
        return InvalidInput::Json(err);
    }
}

#[derive(Debug)]
pub enum NetworkError {
    IO(IOError),
    Request(RequestError),
}

impl Error for NetworkError {}

impl fmt::Display for NetworkError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            NetworkError::IO(err) => {
                write!(formatter, "Streaming Error: {}", err)
            },
            NetworkError::Request(err) => {
                write!(formatter, "Request Error: {}", err)
            }
        }
    }
}

impl From<RequestError> for NetworkError {
    fn from(err: RequestError) -> Self {
        return NetworkError::Request(err);
    }
}

impl From<IOError> for NetworkError {
    fn from(err: IOError) -> Self {
        return NetworkError::IO(err);
    }
}

#[derive(Debug)]
pub enum MissingInput {
    Network(NetworkError),
    File(IOError)
}

impl Error for MissingInput {}

impl fmt::Display for MissingInput {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            MissingInput::Network(err) => {
                write!(formatter, "Network Error: {}", err)
            },
            MissingInput::File(err) => {
                write!(formatter, "Missing File: {}", err)
            }
        };
    }
}

impl From<NetworkError> for MissingInput {
    fn from(err: NetworkError) -> Self {
        return MissingInput::Network(err);
    }
}

impl From<IOError> for MissingInput {
    fn from(err: IOError) -> Self {
        return MissingInput::File(err);
    }
}

#[derive(Debug)]
pub enum InputError {
    Invalid(InvalidInput),
    NoInput(MissingInput),
}

impl Error for InputError {}

impl fmt::Display for InputError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        return match self {
            InputError::Invalid(err) => {
                write!(formatter, "Invalid Input: {}", err)
            },
            InputError::NoInput(err) => {
                write!(formatter, "Missing Input: {}", err)
            }
        };
    }
}

impl From<InvalidInput> for InputError {
    fn from(err: InvalidInput) -> Self {
        return InputError::Invalid(err);
    }
}

impl From<MissingInput> for InputError {
    fn from(err: MissingInput) -> Self {
        return InputError::NoInput(err);
    }
}
