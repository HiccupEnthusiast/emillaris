use serde::{Deserialize, Serialize};

use super::client::ClientInfo;

#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    ClientJoin(ClientInfo),
    ClientList(Vec<ClientInfo>),
    Hello(ClientInfo),
}
