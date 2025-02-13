use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use godot::{
    builtin::{Callable, PackedStringArray, Variant},
    classes::{
        stream_peer_tcp::Status as TcpStatus, timer::TimerProcessCallback,
        web_socket_peer::State as WebSocketState, INode, Node, StreamPeerTcp, TcpServer, Timer,
        WebSocketPeer,
    },
    global::{godot_print, randi_range},
    obj::{Base, Gd, NewAlloc, NewGd, WithBaseField},
    prelude::{godot_api, GodotClass, PackedByteArray},
};
use tracing::{debug, info, warn};

use crate::websocket::default_polling_timer;

use super::{client::ClientInfo, ServerMessage};

struct PendingConnection {
    id: i64,
    connection_time: Instant,
    tcp_stream: Gd<StreamPeerTcp>,
    web_socket: Gd<WebSocketPeer>,
}
impl PendingConnection {
    fn new(id: i64, tcp_stream: Gd<StreamPeerTcp>) -> Self {
        Self {
            id,
            connection_time: Instant::now(),
            web_socket: Self::new_web_socket_peer(&tcp_stream),
            tcp_stream,
        }
    }
    fn new_web_socket_peer(tcp_stream: &Gd<StreamPeerTcp>) -> Gd<WebSocketPeer> {
        let mut res: Gd<WebSocketPeer> = WebSocketPeer::new_gd();
        res.accept_stream(tcp_stream);
        res.set_supported_protocols(&PackedStringArray::from(["emillaris".into()]));
        res
    }
    fn poll(&mut self) -> ConnectionState {
        self.web_socket.poll();
        if self.connection_time.elapsed() > Duration::from_secs(5) {
            return ConnectionState::Disconnected(ConnectionError::Timeout);
        }
        if self.tcp_stream.get_status() != TcpStatus::CONNECTED {
            return ConnectionState::Disconnected(ConnectionError::TcpStreamClosed);
        }
        match self.web_socket.get_ready_state() {
            WebSocketState::OPEN => ConnectionState::Connected(Gd::clone(&self.web_socket)),
            WebSocketState::CONNECTING => ConnectionState::Pending,
            WebSocketState::CLOSING | WebSocketState::CLOSED => {
                ConnectionState::Disconnected(ConnectionError::WebsocketClosed)
            }
            unknown => panic!("unknown state: {unknown:?}"),
        }
    }
}
enum ConnectionState {
    Connected(Gd<WebSocketPeer>),
    Pending,
    Disconnected(ConnectionError),
}
#[derive(Debug, thiserror::Error)]
enum ConnectionError {
    #[error("Timeout")]
    Timeout,
    #[error("Websocket closed")]
    WebsocketClosed,
    #[error("Tcp stream closed")]
    TcpStreamClosed,
}

#[repr(transparent)]
struct ConnectedClients(HashMap<i64, (Gd<WebSocketPeer>, ClientInfo)>);
impl ConnectedClients {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    fn values(&self) -> impl Iterator<Item = &(Gd<WebSocketPeer>, ClientInfo)> {
        self.0.values()
    }
    fn values_mut(&mut self) -> impl Iterator<Item = &mut (Gd<WebSocketPeer>, ClientInfo)> {
        self.0.values_mut()
    }
    fn keep_alive(&mut self) {
        self.0.retain(|_, (_, info)| info.is_alive);
    }
    fn insert_client(&mut self, socket: Gd<WebSocketPeer>, info: ClientInfo) {
        self.0.insert(info.id, (socket, info));
    }
    fn send_message(&mut self, recipient: MessageRecipient, message: ServerMessage) {
        match recipient {
            MessageRecipient::All => {
                for (socket, _) in self.0.values_mut() {
                    let buff = rmp_serde::to_vec(&message).unwrap();
                    let pkt = PackedByteArray::from(buff);
                    socket.send(&pkt);
                }
            }
            MessageRecipient::Include(ids) => {
                for id in ids {
                    if let Some((socket, _)) = self.0.get_mut(id) {
                        let buff = rmp_serde::to_vec(&message).unwrap();
                        let pkt = PackedByteArray::from(buff);
                        socket.send(&pkt);
                    }
                }
            }
            MessageRecipient::Exclude(ids) => {
                let max = self.0.len();
                let mut found = 0;
                for (id, (socket, _)) in self.0.iter_mut() {
                    if found >= max {
                        break;
                    }
                    if !ids.contains(id) {
                        let buff = rmp_serde::to_vec(&message).unwrap();
                        let pkt = PackedByteArray::from(buff);
                        socket.send(&pkt);
                        found += 1;
                    }
                }
            }
        }
    }
}
pub enum MessageRecipient<'a> {
    All,
    Include(&'a [i64]),
    Exclude(&'a [i64]),
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct Server {
    tcp_server: Gd<TcpServer>,
    polling_timer: Gd<Timer>,
    pending_connections: Vec<(PendingConnection, bool)>,
    connected_clients: ConnectedClients,
    counter: i64,
    base: Base<Node>,
}

impl Server {
    #[inline]
    fn poll_for_pending(&mut self) {
        while self.tcp_server.is_connection_available() {
            let tcp_stream = self.tcp_server.take_connection();
            match tcp_stream {
                Some(tcp_stream) => {
                    self.counter = self.counter.wrapping_add(1);
                    let pending = PendingConnection::new(self.counter, tcp_stream);
                    info!(id = pending.id, "Connection incoming");
                    self.pending_connections.push((pending, false));
                }
                None => {
                    warn!("Available connection but failed to take it");
                    continue;
                }
            }
        }
    }
    #[inline]
    fn poll_pending(&mut self) {
        self.pending_connections
            .retain(|(_, is_completed)| !*is_completed);
        for (pending, is_completed) in self.pending_connections.iter_mut() {
            match pending.poll() {
                ConnectionState::Connected(socket) => {
                    let first_join = self.connected_clients.is_empty();
                    let info = ClientInfo {
                        id: pending.id,
                        name: "Hiccu".into(),
                        is_alive: true,
                        is_host: first_join,
                    };
                    self.connected_clients.insert_client(socket, info.clone());
                    *is_completed = true;

                    self.connected_clients.send_message(
                        MessageRecipient::Exclude(&[pending.id]),
                        ServerMessage::ClientJoin(info.clone()),
                    );

                    let mut infos = self
                        .connected_clients
                        .values()
                        .map(|(_, info)| info.clone())
                        .filter(|info| info.id != pending.id)
                        .collect::<Vec<_>>();

                    infos.sort_by_key(|info| info.id);

                    self.connected_clients.send_message(
                        MessageRecipient::Include(&[pending.id]),
                        ServerMessage::ClientList(infos),
                    );

                    self.connected_clients.send_message(
                        MessageRecipient::Include(&[pending.id]),
                        ServerMessage::Hello(info),
                    );

                    info!(id = pending.id, "connection completed");
                }
                ConnectionState::Pending => {
                    debug!(id = pending.id, "connection pending");
                }
                ConnectionState::Disconnected(connection_error) => {
                    warn!(
                        id = pending.id,
                        "connection disconnected: {}", connection_error
                    );
                }
            }
        }
    }
    #[inline]
    fn poll_for_messages(&mut self) {
        self.connected_clients.keep_alive();
        for (socket, info) in self.connected_clients.values_mut() {
            socket.poll();
            if socket.get_ready_state() == WebSocketState::CLOSED {
                info!("{} disconnected", info.id);
                info.is_alive = false;
                continue;
            }
            for _ in 0..socket.get_available_packet_count() {
                let packet = socket.get_packet();
                debug!("{} sent packet: {:?}", info.id, packet);
            }
        }
    }
}
#[godot_api]
impl Server {
    #[func]
    fn poll(&mut self) {
        if !self.tcp_server.is_listening() {
            warn!("Server is not listening");
            return;
        }
        self.poll_for_pending();
        self.poll_pending();
        self.poll_for_messages();
    }
}

#[godot_api]
impl INode for Server {
    fn init(base: Base<Node>) -> Self {
        let tcp_server = TcpServer::new_gd();
        let polling_timer = super::default_polling_timer();

        Self {
            tcp_server,
            polling_timer,
            pending_connections: Vec::new(),
            connected_clients: ConnectedClients(HashMap::new()),
            counter: 0,
            base,
        }
    }
    fn ready(&mut self) {
        {
            let timer = self.polling_timer.clone();
            self.base_mut().add_child(&timer);
        }
        info!("Starting server on port 8000");
        self.tcp_server.listen(8000);
        let callable = self.base().callable("poll");
        self.polling_timer.connect("timeout", &callable);
    }
}
