use std::error::Error;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::mpsc::{Receiver, Sender};
use log::{debug, warn};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream};
use tokio::runtime::{Builder, Runtime};
use brydz_core::error::BridgeErrorStd;
use brydz_core::protocol::{ClientDealMessage, ServerDealMessage};
use brydz_core::speedy;
use crate::tcp::TcpForwardError;
use crate::tcp::TcpForwardError::StreamSendError;
use brydz_core::speedy::{LittleEndian, Writable};
use brydz_core::world::comm::{CommunicationEnd, CommunicationEndStd};


pub struct TokioTcpComm{
    stream: TcpStream,
    buffer: [u8; 256],
    rt: Runtime
}

impl TokioTcpComm{
    pub fn new(stream: TcpStream) -> Self{
        let rt = Builder::new_current_thread().build().unwrap();
        Self{stream, buffer: [0; 256], rt }
    }
}

impl<OT, IT, E: Error> CommunicationEnd<OT, IT, E> for TokioTcpComm
where OT: brydz_core::serde::Serialize, IT: brydz_core::serde::Deserialize{
    fn send(&self, message: OT) -> Result<(), E> {
        //message.
        //self.rt.block_on(|| self.stream.try_write())
        todo!();
    }

    fn recv(&mut self) -> Result<IT, E> {
        todo!()
    }

    fn try_recv(&mut self) -> Result<IT, E> {
        todo!()
    }
}

pub struct TcpComm{
    stream: std::net::TcpStream
}