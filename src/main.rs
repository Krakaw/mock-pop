use mio::event::Event;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Registry, Token};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::str::from_utf8;

const SERVER: Token = Token(0);

const OK_RESPONSE: &str = "+OK";
const ERR_RESPONSE: &str = "-ERR";

fn main() -> io::Result<()> {
    let list = Arc::new(List::new());
// Create a poll instance.
    let mut poll = Poll::new()?;
    // Create storage for events.
    let mut events = Events::with_capacity(128);

    // Setup the TCP server socket.
    let addr = "127.0.0.1:9000".parse().unwrap();
    let mut server = TcpListener::bind(addr)?;

    // Register the server with poll we can receive events for it.
    poll.registry()
        .register(&mut server, SERVER, Interest::READABLE)?;

    // Map of `Token` -> `TcpStream`.
    let mut connections = HashMap::new();
    // Unique token for each incoming connection.
    let mut unique_token = Token(SERVER.0 + 1);

    println!("You can connect to the server using `nc`:");
    println!(" $ nc 127.0.0.1 9000");
    println!("You'll see our welcome message and anything you type we'll be printed here.");
    loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            match event.token() {
                SERVER => loop {
                    // Received an event for the TCP server socket, which
                    // indicates we can accept an connection.
                    let (mut connection, address) = match server.accept() {
                        Ok((connection, address)) => (connection, address),
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            // If we get a `WouldBlock` error we know our
                            // listener has no more incoming connections queued,
                            // so we can return to polling and wait for some
                            // more.
                            break;
                        }
                        Err(e) => {
                            // If it was any other kind of error, something went
                            // wrong and we terminate with an error.
                            return Err(e);
                        }
                    };

                    println!("Accepted connection from: {}", address);

                    let token = next(&mut unique_token);
                    poll.registry().register(
                        &mut connection,
                        token,
                        Interest::READABLE.add(Interest::WRITABLE),
                    )?;

                    connections.insert(token, connection);
                },
                token => {
                    // Maybe received an event for a TCP connection.
                    let done = if let Some(connection) = connections.get_mut(&token) {
                        handle_connection_event(poll.registry(), connection, event)?
                    } else {
                        // Sporadic events happen, we can safely ignore them.
                        false
                    };
                    if done {
                        connections.remove(&token);
                    }
                }
            }
        }
    }
}

fn next(current: &mut Token) -> Token {
    let next = current.0;
    current.0 += 1;
    Token(next)
}

/// Returns `true` if the connection is done.
fn handle_connection_event(
    registry: &Registry,
    connection: &mut TcpStream,
    event: &Event,
) -> io::Result<bool> {
    let mut messages = vec!["Message One".to_string(), "Message 2".to_string()];
    if event.is_writable() {
        // We can (maybe) write to the connection.
        match connection.write(b"+OK mock-pop POP3 service is ready.\n") {
            // We want to write the entire `DATA` buffer in a single go. If we
            // write less we'll return a short write error (same as
            // `io::Write::write_all` does).
            Ok(n) if n < OK_RESPONSE.len() => return Err(io::ErrorKind::WriteZero.into()),
            Ok(_) => {
                // After we've written something we'll reregister the connection
                // to only respond to readable events.
                registry.reregister(connection, event.token(), Interest::READABLE)?
            }
            // Would block "errors" are the OS's way of saying that the
            // connection is not actually ready to perform this I/O operation.
            Err(ref err) if would_block(err) => {}
            // Got interrupted (how rude!), we'll try again.
            Err(ref err) if interrupted(err) => {
                return handle_connection_event(registry, connection, event);
            }
            // Other errors we'll consider fatal.
            Err(err) => return Err(err),
        }
    }

    if event.is_readable() {
        let mut connection_closed = false;
        let mut received_data = Vec::with_capacity(4096);
        // We can (maybe) read from the connection.
        loop {
            let mut buf = [0; 256];
            match connection.read(&mut buf) {
                Ok(0) => {
                    // Reading 0 bytes means the other side has closed the
                    // connection or is done writing, then so are we.
                    connection_closed = true;
                    break;
                }
                Ok(n) => {
                    received_data.extend_from_slice(&buf[..n]);
                    let r = received_data.clone();

                    match from_utf8(&r) {
                        Ok(str_buf) => {

                            let parts = str_buf.trim().split("\n").map(|p| p.trim()).collect::<Vec<&str>>();
                            for part in parts {
                                println!("Processing '{}'", part);
                                let commands = part.split(" ").collect::<Vec<&str>>();
                                // https://ec2.freesoft.org/CIE/RFC/1725/9.htm
                                let response = match commands[0].to_uppercase().as_str() {
                                    "USER" | "PASS"  | "NOOP" | "RSET" | "QUIT" | "UIDL" => Ok("\n".to_string()),
                                    "STAT" => {
                                        let len = messages.len();
                                        let size = messages.iter().fold(0, |a,b|a + b.as_bytes().len());
                                        Ok(format!("{} {}\n", len, size))
                                    },//count, bytes
                                    "LIST" => {
                                        let len = messages.len();
                                        let size = messages.iter().fold(0, |a,b|a + b.as_bytes().len());
                                        let messages = messages.iter().enumerate().map(|(i, val)| format!("{} {}", i + 1, val.as_bytes().len())).collect::<Vec<String>>().join("\n");
                                        Ok(format!("{} messages ({} octects)\n{}\n.\n", len, size, messages)) //The 1 size is increment and size of messages
                                    }
                                    "RETR" =>{
                                        let opt:usize = commands.get(1).unwrap().parse().unwrap();
                                        let message = messages.get(opt -1).unwrap().clone();

                                        Ok(format!("{} octets\n{}\n.\n", message.clone().as_bytes().len(), message))
                                    },//Send message
                                    "DELE" => {
                                        let opt:usize = commands.get(1).unwrap().parse().unwrap();
                                        messages.remove(opt -1);
                                        println!("{:?}", messages);
                                        Ok(format!("message {} deleted\n", opt))
                                    },
                                    "CAPA" => Ok("\nPLAIN\n.\n".to_string()),
                                    "AUTH" => Ok("\nPLAIN\nANONYMOUS\n.\n".to_string()),
                                    _ => Ok("\n".to_string())
                                };
                                let txt = match response.map(|s| format!("{} {}", OK_RESPONSE, s)).map_err(|e: String| format!("{} {}", ERR_RESPONSE, e)) {
                                    Ok(s) => s,
                                    Err(s) => s
                                };
                                println!("Responding: {}", txt);
                                connection.write(txt.as_bytes());
                            }
                        }
                        Err(_) => {}
                    };


                    break;
                }
                // Would block "errors" are the OS's way of saying that the
                // connection is not actually ready to perform this I/O operation.
                Err(ref err) if would_block(err) => break,
                Err(ref err) if interrupted(err) => continue,
                // Other errors we'll consider fatal.
                Err(err) => return Err(err),
            }
        }

        if let Ok(str_buf) = from_utf8(&received_data) {
            println!("Received data: {}", str_buf.trim_end());
        } else {
            println!("Received (none UTF-8) data: {:?}", &received_data);
        }

        if connection_closed {
            println!("Connection closed");
            return Ok(true);
        }
    }

    Ok(false)
}

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

fn interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}
