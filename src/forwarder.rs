use std::error::Error;


pub trait Forwarder{
    type Error: Error;
    type Message;

    fn send(&self, message: &Self::Message) -> Result<(), Self::Error>;
    fn recv(&self) -> Result<Self::Message, Self::Error>;
    fn try_recv(&self) -> Result<Self::Message, Self::Error>;
}