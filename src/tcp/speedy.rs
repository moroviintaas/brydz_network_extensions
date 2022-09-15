use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::mpsc::{Receiver, Sender};
use log::{debug, warn};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use brydz_core::speedy;
use crate::tcp::TcpForwardError;
use crate::tcp::TcpForwardError::StreamSendError;
use brydz_core::speedy::LittleEndian;

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