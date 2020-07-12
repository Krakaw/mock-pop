use bytes::{Buf, BytesMut};
use clap::Clap;
use futures::SinkExt;
use log::{debug, info, trace, warn};
use std::fmt::Display;
use std::net::{IpAddr, SocketAddr};
use std::{env, error::Error, fmt, io};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
use tokio_util::codec::{Decoder, Encoder, Framed};

#[derive(Clap)]
#[clap(
    version = "0.1",
    author = "Krakaw <41575888+Krakaw@users.noreply.github.com>"
)]
struct Opts {
    /// Listening address
    #[clap(short, long, default_value = "127.0.0.1:1100")]
    listen: SocketAddr,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let opts: Opts = Opts::parse();
    let addr = opts.listen;
    let mut server = TcpListener::bind(addr.to_string()).await?;
    let mut incoming = server.incoming();
    info!("Listening on: {}", addr);

    while let Some(Ok(mut stream)) = incoming.next().await {
        stream.write(b"+OK Welcome to mock-pop\n").await;
        tokio::spawn(async move {
            if let Err(e) = process(stream).await {
                warn!("failed to process connection; error = {}", e);
            }
        });
    }

    Ok(())
}

async fn process(stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut transport = Framed::new(stream, Pop);

    while let Some(request) = transport.next().await {
        debug!("Request {:?}", request);
        match request {
            Ok(request) => {
                let responses = respond(request).await?;
                trace!("Got vec of responses: {:?}", responses);
                for response in responses {
                    transport.send(response).await?;
                }
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}

async fn respond(requests: Vec<Request>) -> Result<Vec<Response>, Box<dyn Error>> {
    let mut responses = vec![];
    for request in requests {
        responses.push(request.into())
    }
    Ok(responses)
}

#[derive(Debug)]
pub enum Command {
    User(Option<String>),
    Pass(Option<String>),
    Noop,
    Rset,
    Quit,
    Uidl,
    Stat,
    List,
    Retr(u32),
    Dele(u32),
    Capa,
    Auth,
}

impl Command {
    pub fn from_str(s: &str) -> Option<Command> {
        let parts = s.trim().to_uppercase();
        let parts = parts.split(" ").collect::<Vec<&str>>();
        if parts.is_empty() {
            return None;
        }
        match parts[0] {
            "USER" => Some(Command::User(parts.get(1).map(|s| s.to_string()))),
            "PASS" => Some(Command::Pass(parts.get(1).map(|s| s.to_string()))),
            "NOOP" => Some(Command::Noop),
            "RSET" => Some(Command::Rset),
            "QUIT" => Some(Command::Quit),
            "UIDL" => Some(Command::Uidl),
            "STAT" => Some(Command::Stat),
            "LIST" => Some(Command::List),
            "RETR" => Some(Command::Retr(
                parts.get(1).map(|i| i.parse().unwrap_or(0)).unwrap_or(0),
            )),
            "DELE" => Some(Command::Dele(
                parts.get(1).map(|i| i.parse().unwrap_or(0)).unwrap_or(0),
            )),
            "CAPA" => Some(Command::Capa),
            "AUTH" => Some(Command::Auth),
            _ => None,
        }
    }

    pub fn respond(&self) -> String {
        match self {
            Command::User(a) => format!("User: {:?}", a.as_ref().unwrap_or(&"".to_string())),
            Command::Pass(a) => format!("Pass: {:?}", a.as_ref().unwrap_or(&"".to_string())),
            Command::Stat => {
                let messages: Vec<String> = vec![];
                let message_count = messages.len();
                let message_size = messages.iter().fold(0, |a, b| a + b.as_bytes().len());
                format!("{} {}", message_count, message_size)
            }
            Command::List => {
                let messages: Vec<String> = vec![];
                let message_count = messages.len();
                let message_size = messages.iter().fold(0, |a, b| a + b.as_bytes().len());
                let message_list = messages
                    .iter()
                    .enumerate()
                    .map(|(i, val)| format!("{} {}", i + 1, val.as_bytes().len()))
                    .collect::<Vec<String>>()
                    .join("\n");
                format!(
                    "{} messages ({} octets)\n{}\n.",
                    message_count, message_size, message_list
                )
            }
            Command::Retr(message_index) => {
                let message = "abcd";
                let message_size = message.clone().as_bytes().len();

                format!("{} octets\n{}\n.", message_size, message)
            }
            Command::Dele(message_index) => format!("message {} deleted", message_index),
            Command::Capa => "\nPLAIN\n.".to_string(),
            Command::Auth => "\nPLAIN\nANONYMOUS\n.".to_string(),
            _ => "".to_string(),
        }
    }
}

struct Pop;
#[derive(Debug)]
struct Request {
    pub command: Option<Command>,
}
#[derive(Debug)]
struct Response {
    pub command: Option<Command>,
}

impl From<Request> for Response {
    fn from(request: Request) -> Self {
        Response {
            command: request.command,
        }
    }
}

impl Response {
    pub fn respond(&self) -> String {
        if self.command.is_none() {
            return String::from("-ERR Invalid command\n");
        }

        let command = self.command.as_ref().unwrap();
        format!("+OK {}\n", command.respond())
    }
}

impl Encoder<Response> for Pop {
    type Error = io::Error;
    fn encode(&mut self, item: Response, dst: &mut BytesMut) -> io::Result<()> {
        use std::fmt::Write;
        debug!("Responding {:?}", item.respond());
        dst.extend_from_slice(item.respond().as_bytes());
        Ok(())
    }
}

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
