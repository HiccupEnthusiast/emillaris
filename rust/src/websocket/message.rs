use serde::{Deserialize, Serialize};

use super::client::ClientInfo;

#[derive(Debug, Deserialize, Serialize)]
pub enum ServerMessage {
    ClientJoin(ClientInfo),
    ClientList(Vec<ClientInfo>),
    Hello(ClientInfo),
}
