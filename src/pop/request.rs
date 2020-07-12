use crate::pop::command::Command;
use crate::pop::response::Response;
use bytes::BytesMut;
use log::{debug, trace};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug)]
pub struct Request {
    pub command: Option<Command>,
}

pub struct Pop;
impl Decoder for Pop {
    type Item = Vec<Request>;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }
        let buf = src.split();
        trace!("Decoding buf: {:?}", buf.clone());
        let requests = String::from_utf8_lossy(buf.as_ref())
            .trim()
            .split("\n")
            .map(|p| {
                let command = Command::from_str(p.trim());
                Request { command }
            })
            .collect::<Vec<Request>>();

        return Ok(Some(requests));
    }
}

impl Encoder<Response> for Pop {
    type Error = io::Error;
    fn encode(&mut self, item: Response, dst: &mut BytesMut) -> io::Result<()> {
        debug!("Responding {:?}", item.respond());
        dst.extend_from_slice(item.respond().as_bytes());
        Ok(())
    }
}
