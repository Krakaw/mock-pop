use crate::pop::request::{Pop, Request};
use crate::pop::response::Response;
use futures::SinkExt;
use log::{debug, info, trace, warn};
use std::error::Error;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
use tokio_util::codec::Framed;

pub async fn start(addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    let mut server = TcpListener::bind(addr.to_string()).await?;
    let mut incoming = server.incoming();
    info!("Listening on: {}", addr);

    while let Some(Ok(mut stream)) = incoming.next().await {
        let _ = stream.write(b"+OK Welcome to mock-pop\n").await;
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
