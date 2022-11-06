use std::error::Error;
use std::fmt::Debug;
use std::io::{Write, Read};
use std::marker::PhantomData;
use std::sync::mpsc::{Receiver, Sender};
use log::{debug, warn};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream};
use brydz_framework::brydz_core::error::{ FormatError};
use brydz_framework::brydz_core::speedy;
use crate::tcp::TcpForwardError;
use crate::tcp::TcpForwardError::StreamSendError;
use brydz_framework::brydz_core::speedy::{ LittleEndian, Readable, Writable};
use brydz_framework::error::comm::CommError;
use brydz_framework::world::comm::{CommunicationEnd};


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
/*
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

impl<'a, OT, IT, E: Error> CommunicationEnd for TokioTcpComm
where OT: Writable<LittleEndian> + Debug, IT: Readable<'a, LittleEndian> + Debug,
E: From<CommError>{
    fn send(&mut self, message: OT) -> Result<(), E> {
        let mut buffer = [0u8;1024];
        message.write_to_buffer(&mut buffer).unwrap(); //to impl E: From<speedy::Error>

        self.rt.block_on(async {
            if self.stream.writable().await.is_err(){
                warn!("Error waiting for TCP socket to be ready to write.");
            }
            
        });
        self.stream.try_write(&buffer).map_or_else(|_| {Err(CommError::TrySendError.into())}, |_| { Ok(())})

    }

    fn recv(&mut self) -> Result<IT, E> {
        let mut buffer = [0u8;1024];
        self.rt.block_on(async  {
            if self.stream.readable().await.is_err(){
                warn!("Error waiting for TCP socket to be ready to read.")
            }
            


        });
        let mut completed = false;
            while !completed{
                match self.stream.try_read(&mut buffer){
                    Ok(_) => {completed = true;}
                    Err(_) => {return Err(CommError::RecvError.into());}
                }
            }
        debug!("Received speedy data: {:?}", &buffer);
        let it = IT::read_from_buffer_copying_data(&buffer).unwrap();
        info!("Received data: {:?}", it);
        Ok(it)

    }

    fn try_recv(&mut self) -> Result<IT, E> {
        let mut buffer = [0u8;1024];
        self.rt.block_on(async  {
            if self.stream.readable().await.is_err(){
                warn!("Error waiting for TCP socket to be ready to read.")
            }
            


        });

        if self.stream.try_read(&mut buffer).is_err(){
            return Err(CommError::TryRecvError.into());
        }

        debug!("Received speedy data: {:?}", &buffer);
        let it = IT::read_from_buffer_copying_data(&buffer).unwrap();
        info!("Received data: {:?}", it);
        Ok(it)
    }

    type OutputType = OT;

    type InputType = IT;

    type Error = E;
}
*/

pub struct TcpComm<OT, IT, E: Error>{
    stream: std::net::TcpStream,
    _ot: PhantomData<OT>,
    _it: PhantomData<IT>,
    _e: PhantomData<E>
}

impl<OT, IT, E: Error> TcpComm<OT, IT, E>{
    pub fn new(stream : std::net::TcpStream) -> Self{
        Self{stream, _ot: PhantomData::default(), _it: PhantomData::default(), _e: PhantomData::default()}
    }
}

//this need to be removed in future, size of buffer should be provided by generic paramaters <OT> and <IT> in CommunictaionEnd's impl
const BRIDGE_COMM_BUFFER_SIZE: usize = 256; 

impl<'a, OT, IT, E: Error> CommunicationEnd for TcpComm<OT, IT, E>
where OT: Writable<LittleEndian> + Debug, IT: Readable<'a, LittleEndian> + Debug,
E: From<CommError> + From<FormatError>{
    fn send(&mut self, message: OT) -> Result<(), E> {
        
        let mut buffer = [0u8; BRIDGE_COMM_BUFFER_SIZE];
        if message.write_to_buffer(&mut buffer).is_err(){
            return Err(FormatError::SerializeError.into())
        }
        match self.stream.write_all(&buffer){
            Ok(_) => Ok(()),
            Err(_) => Err(CommError::SendError.into()),
        }
    }

    fn recv(&mut self) -> Result<IT, E> {
        self.stream.set_nonblocking(false).unwrap();
        let mut buffer = [0u8; BRIDGE_COMM_BUFFER_SIZE];
        let mut received = false;
        while !received {
            match self.stream.read(&mut buffer){
                Ok(0) => {},
                Ok(_) => {
                    received = true;
                },
                Err(_e) => {return Err(CommError::RecvError.into())}
            }
        }
        match IT::read_from_buffer_copying_data(&buffer){
            Ok(m) => Ok(m),
            Err(_) => Err(FormatError::DeserializeError.into())
        }
    }

    fn try_recv(&mut self) -> Result<IT, E> {
        let mut buffer = [0u8; BRIDGE_COMM_BUFFER_SIZE];
        self.stream.set_nonblocking(true).unwrap();
        
        match self.stream.read(&mut buffer){
            Ok(0) => {
                //debug!("TryRecvError");
                Err(CommError::TryRecvError.into())
            }
            Ok(_n) => {
                //debug!("Tcp TryRecv with {} bytes", n);
                match IT::read_from_buffer_copying_data(&buffer){
                    Ok(m) => Ok(m),
                    Err(_) => Err(FormatError::DeserializeError.into())
                }
            },
            Err(_e) => {
                
                Err(CommError::TryRecvError.into())
            }
        }
        
        
    }

    type OutputType = OT;

    type InputType = IT;

    type Error = E;

}