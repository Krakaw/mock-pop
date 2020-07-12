use crate::pop::command::Command;
use crate::pop::request::Request;

#[derive(Debug)]
pub struct Response {
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
