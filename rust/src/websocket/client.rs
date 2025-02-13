use godot::{
    builtin::{Dictionary, PackedStringArray},
    classes::{web_socket_peer::State as WebSocketState, INode, Node, Timer, WebSocketPeer},
    meta::{FromGodot, ToGodot},
    obj::{Base, Gd, WithBaseField},
    prelude::{godot_api, ConvertError, GodotClass, GodotConvert},
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::websocket::ServerMessage;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ClientInfo {
    pub id: i64,
    pub name: String,
    pub is_alive: bool,
    pub is_host: bool,
}

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct Client {
    pub info: ClientInfo,
    socket: Gd<WebSocketPeer>,
    #[init(val = super::default_polling_timer())]
    polling_timer: Gd<Timer>,
    base: Base<Node>,
}

impl Client {
    #[inline]
    fn handle_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::ClientJoin(client_info) => {
                self.base_mut()
                    .emit_signal("client_joined", &[client_info.to_variant()]);
            }
            ServerMessage::ClientList(client_infos) => {
                for info in client_infos {
                    self.base_mut()
                        .emit_signal("client_joined", &[info.to_variant()]);
                }
            }
            ServerMessage::Hello(client_info) => {
                self.base_mut()
                    .emit_signal("connection_established", &[client_info.to_variant()]);
                self.info = client_info;
            }
        }
    }
}

#[godot_api]
impl Client {
    #[func]
    fn poll(&mut self) {
        if self.socket.get_ready_state() != WebSocketState::CLOSED {
            self.socket.poll();
        }
        self.info.is_alive = self.socket.get_ready_state() == WebSocketState::OPEN;
        if self.info.is_alive {
            for _ in 0..self.socket.get_available_packet_count() {
                let packet = self.socket.get_packet();
                match rmp_serde::from_slice::<ServerMessage>(packet.as_slice()) {
                    Ok(message) => {
                        debug!(name = self.info.name, "Received message: {:?}", message);
                        self.handle_message(message);
                    }
                    Err(e) => error!(
                        name = self.info.name,
                        "failed to deserialize packet: {}\n\t{}",
                        e,
                        String::from_utf8_lossy(packet.as_slice())
                    ),
                }
            }
        }
    }
    #[signal]
    fn client_joined(info: ClientInfo);
    #[signal]
    fn connection_established(info: ClientInfo);
}

#[godot_api]
impl INode for Client {
    fn ready(&mut self) {
        {
            let timer = self.polling_timer.clone();
            self.base_mut().add_child(&timer);
        }
        let callable = self.base().callable("poll");
        self.polling_timer.connect("timeout", &callable);
        self.socket
            .set_supported_protocols(&PackedStringArray::from(["emillaris".into()]));
        self.socket.connect_to_url("ws://localhost:8000");
    }
}

impl GodotConvert for ClientInfo {
    type Via = Dictionary;
}
impl FromGodot for ClientInfo {
    fn try_from_godot(via: Self::Via) -> Result<Self, ConvertError> {
        let id = via
            .get("id")
            .ok_or(ConvertError::new("Missing id"))?
            .try_to()?;
        let name = via
            .get("name")
            .ok_or(ConvertError::new("Missing name"))?
            .try_to()?;
        let is_alive = via
            .get("is_alive")
            .ok_or(ConvertError::new("Missing is_alive"))?
            .try_to()?;
        let is_host = via
            .get("is_host")
            .ok_or(ConvertError::new("Missing is_host"))?
            .try_to()?;
        debug!(name, "Deserialized from godot");
        Ok(ClientInfo {
            id,
            name,
            is_alive,
            is_host,
        })
    }
}
impl ToGodot for ClientInfo {
    type ToVia<'v> = Self::Via;
    fn to_godot(&self) -> Self::ToVia<'_> {
        debug!(name = self.name, "Serialized to godot");
        let mut dict = Dictionary::new();
        dict.set("id", self.id);
        dict.set("name", self.name.clone());
        dict.set("is_alive", self.is_alive);
        dict.set("is_host", self.is_host);
        dict
    }
}
