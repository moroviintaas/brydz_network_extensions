use std::io::Error;
use std::sync::mpsc::{RecvError, SendError};

pub mod speedy;
//pub mod ron;

#[derive( Debug)]
pub enum TcpForwardError{
    ChannelRecvError,
    ChannelSendError,
    StreamRecvError(tokio::io::Error),
    StreamSendError(tokio::io::Error),
    IOError(tokio::io::Error),
    StreamSendRemoteClosed,
    StreamRecvRemoteClosed,
    SerializeError,
    DeserializeError,

}

impl From<RecvError> for TcpForwardError{
    fn from(_: RecvError) -> Self {
        Self::ChannelRecvError
    }
}

impl<T> From<SendError<T>> for TcpForwardError{
    fn from(_: SendError<T>) -> Self {
        Self::ChannelSendError
    }
}

impl From<tokio::io::Error> for TcpForwardError{
    fn from(e: Error) -> Self {
        Self::IOError(e)
    }
}


