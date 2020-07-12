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
                let messages: Vec<&str> = vec!["abcd"];
                let message = messages.get(*message_index as usize).unwrap_or(&"");
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
