use std::error::Error;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::mpsc::{Receiver, Sender};
use log::{debug, warn, info};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream};
use tokio::runtime::{Builder, Runtime};
use brydz_core::error::BridgeErrorStd;
use brydz_core::protocol::{ClientDealMessage, ServerDealMessage};
use brydz_core::speedy;
use crate::tcp::TcpForwardError;
use crate::tcp::TcpForwardError::StreamSendError;
use brydz_core::speedy::{Context, LittleEndian, Readable, Writable};
use brydz_core::world::comm::{CommunicationEnd, CommunicationEndStd};

pub struct TcpSpeedyForwarder<'a, RT: speedy::Readable<'a, LittleEndian> + Debug, WT: speedy::Writable<LittleEndian> + Debug> {
    read_stream: OwnedReadHalf,
    write_stream: OwnedWriteHalf,

    read_channel: Receiver<WT>,
    write_channel: Sender<RT>,
    phantom: PhantomData<&'a LittleEndian>


}

impl<'a, RT: Debug +  speedy::Readable<'a, LittleEndian>, WT: Debug + speedy::Writable<LittleEndian>> TcpSpeedyForwarder<'a, RT, WT>{

    pub fn new(read_channel: Receiver<WT>, write_channel: Sender<RT>, stream: TcpStream) -> Self{
        let (read_stream, write_stream) = stream.into_split();
        Self{
            read_channel, write_channel, read_stream, write_stream,
            phantom: Default::default()
        }
    }

    pub async fn run(&self) -> Result<(), TcpForwardError>{
        debug!("Starting channel/tcp forwarder.");
        let mut stream_received = Vec::new();
        //let mut b = Vec::new();
        loop{
            //check on channel
            if let Ok(channel_received) = self.read_channel.try_recv(){
                debug!("Received message {:?} ", &channel_received);
                let serialization = match channel_received.write_to_vec(){
                    Ok(message) => {message}
                    Err(_e) => {
                        warn!("TcpForwardError::SerializeError");
                        continue
                    }
                };
                self.write_stream.writable().await?;
                match self.write_stream.try_write(&serialization){
                    Ok(_) => {}
                    Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock=> { continue},
                    Err(e) =>{
                        return Err(StreamSendError(e));
                    }
                }
            }
            self.read_stream.readable().await?;
            let _n = match self.read_stream.try_read_buf(&mut stream_received){
                Ok(0) => {
                    return Err(TcpForwardError::StreamRecvRemoteClosed);
                }
                Ok(n) => n,
                Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => { continue},
                Err(e) => { return Err(TcpForwardError::StreamRecvError(e))}
            };
            //b = stream_received.clone();
            let message = match RT::read_from_buffer_copying_data(&stream_received) {
                Ok(x) => x,
                Err(e) => {
                    warn!("Deserialization error: {}", e);
                    continue;
                }
            };
            self.write_channel.send(message)?;



        }
    }
}

pub struct TokioTcpComm{
    stream: TcpStream,
    rt: Runtime
}

impl TokioTcpComm{
    pub fn new(stream: TcpStream) -> Self{
        let rt = Builder::new_current_thread().build().unwrap();
        Self{stream,  rt }
    }
}
/*
impl<'a, OT, IT, E: Error, C: Context> CommunicationEnd<OT, IT, E> for TokioTcpComm
where OT: Writable<C> + Debug, IT: Readable<'a, C> + Debug{
    fn send(&self, message: OT) -> Result<(), E> {
        let mut buffer = [0u8;1024];
        message.write_to_buffer(&mut buffer).unwrap(); //to impl E: From<speedy::Error>

        self.rt.block_on(|| {
            self.stream.writable().unwrap();
            self.stream.try_write(&buffer).unwrap()
        })
    }

    fn recv(&mut self) -> Result<IT, E> {
        let mut buffer = [0u8;1024];
        self.rt.block_on(|| {
            self.stream.readable();
            let mut completed = false;
            while !completed{
                match self.stream.try_read_buf(&mut buffer){
                    Ok(_) => {completed = true;}
                    Err(_) => {}
                }
            }


        });
        debug!("Received speedy data: {:?}", &buffer);
        let it = IT::read_from_buffer_copying_data(&buffer).unwrap();
        info!("Received data: {:?}", it);
        Ok(it)

    }

    fn try_recv(&mut self) -> Result<IT, E> {
        todo!()
    }
}
*/

pub struct TcpComm{
    stream: std::net::TcpStream
}