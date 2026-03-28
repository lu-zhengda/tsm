use std::fmt;

#[derive(Debug)]
pub enum Error {
    Connection(String),
    Auth,
    SessionExpired,
    Rpc(String),
    Config(String),
    TorrentNotFound(String),
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Connection(msg) => write!(f, "{msg}"),
            Error::Auth => write!(
                f,
                "Authentication failed. Check your username and password."
            ),
            Error::SessionExpired => write!(f, "Session expired. Please retry."),
            Error::Rpc(msg) => write!(f, "RPC error: {msg}"),
            Error::Config(msg) => write!(f, "Configuration error: {msg}"),
            Error::TorrentNotFound(id) => write!(f, "Torrent not found: {id}"),
            Error::Io(err) => write!(f, "IO error: {err}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::Connection(_) | Error::Auth | Error::SessionExpired | Error::Rpc(_) => 1,
            Error::Config(_) => 2,
            Error::TorrentNotFound(_) => 3,
            Error::Io(_) => 1,
        }
    }
}
