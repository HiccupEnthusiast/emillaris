extends Node
class_name Server

enum CONNECTION_STATUS {
	CONNECTED,
	PENDING,
	DISCONNECTED
}

class PendingConnection:
	var id: int
	var connection_time: int
	var tcp_stream: StreamPeerTCP
	var websocket: WebSocketPeer

	func _init(stream: StreamPeerTCP) -> void:
		id = randi_range(2, 1 << 30)
		connection_time = Time.get_ticks_msec() 
		tcp_stream = stream

	func try_connect() -> ConnectionResult:
		if Time.get_ticks_msec() - connection_time > 5000:
			Log.warn("Connection %d timed out, dropping" % [id], Log.SCOPE_SERVER)
			return ConnectionResult.failed()
		if websocket != null:
			websocket.poll()
			var state = websocket.get_ready_state()

			if state == WebSocketPeer.STATE_OPEN:
				Log.info("Connection %d opened" % [id], Log.SCOPE_SERVER)
				return ConnectionResult.success(websocket)
			elif state == WebSocketPeer.STATE_CONNECTING:
				Log.info("Connection %d is still connecting" % [id], Log.SCOPE_SERVER)
				return ConnectionResult.still_pending()
			else:
				Log.warn("Connection %d's websocket closed" % [id], Log.SCOPE_SERVER)
				return ConnectionResult.failed()
		elif tcp_stream.get_status() != StreamPeerTCP.STATUS_CONNECTED:
			Log.warn("Connection %d's tcp stream closed" % [id], Log.SCOPE_SERVER)
			return ConnectionResult.failed()
		else: 
			Log.info("Initiating websocket connection for connection %d" % [id], Log.SCOPE_SERVER)
			# TODO: Implement TLS
			websocket = WebSocketPeer.new()
			websocket.supported_protocols = ["emillaris"]
			websocket.accept_stream(tcp_stream)
			return ConnectionResult.still_pending()

class ConnectionResult:
	var websocket: WebSocketPeer
	var status: CONNECTION_STATUS

	static func failed() -> ConnectionResult:
		var result = ConnectionResult.new()
		result.status = CONNECTION_STATUS.DISCONNECTED
		return result

	static func still_pending() -> ConnectionResult:
		var result = ConnectionResult.new()
		result.status = CONNECTION_STATUS.PENDING
		return result

	static func success(peer: WebSocketPeer) -> ConnectionResult:
		var result = ConnectionResult.new()
		result.websocket = peer
		result.status = CONNECTION_STATUS.CONNECTED
		return result


var pending_connections: Array[PendingConnection]
var connected_peers: Dictionary
var tcp_server := TCPServer.new()
var polling_timer := Timer.new()

func poll() -> void:
	# Poll pending connections
	if not tcp_server.is_listening():
		Log.warn("Server is not listening, skipping polling", Log.SCOPE_SERVER)
		return

	while tcp_server.is_connection_available():
		var stream = tcp_server.take_connection()
		if stream == null:
			Log.warn("Tcp server has connection available but failed to take it", Log.SCOPE_SERVER)
			continue
		pending_connections.append(PendingConnection.new(stream))

	# Try to drive pending to actual connections
	var to_remove = []
	for connection in pending_connections:
		var result = connection.try_connect()

		if result.status == CONNECTION_STATUS.CONNECTED:
			var first_join = connected_peers.size() == 0
			var c_info = ClientInfo.new(connection.id, "HI", first_join)

			connected_peers[connection.id] = {
				"client_info": c_info,
				"socket": result.websocket
				}

			# Send the new client to the rest of the clients 
			send_message([connection.id], Message.player_joined(c_info), true)

			# Send the rest of the client to the new client 
			for id in connected_peers:
				if id != connection.id:
					send_message([connection.id], Message.player_joined(connected_peers[id].client_info), false)

			# Send hello message
			send_message([connection.id], Message.hello(c_info), false)


			to_remove.append(connection)
		elif result.status == CONNECTION_STATUS.DISCONNECTED:
			to_remove.append(connection)

	for connection in to_remove:
		pending_connections.erase(connection)
	
	# Poll connections for messages
	to_remove = []
	for id in connected_peers.keys():
		var peer = connected_peers[id].socket
		peer.poll()
		if peer.get_ready_state() == WebSocketPeer.STATE_CLOSED:
			to_remove.append(id)
			continue

		while peer.get_available_packet_count() > 0:
			var packet = peer.get_packet()
			Log.debug("Conection %d sent message: %s" % [id, packet.get_string_from_utf8()], Log.SCOPE_SERVER)

	for id in to_remove:
		connected_peers.erase(id)

# Sends a message to peers, and empty array send to all, exclude dictates if the array is a white or black list
func send_message(ids: Array[int], message: Message, exclude := false) -> void:
	var fn = func(cid: int):
		connected_peers[cid].socket.send_text(message.serialize())
	if ids.size() == 0:
		for cid in connected_peers:
			fn.call(cid)
		return
	for id in ids:
		for cid in connected_peers:
			# XOR
			if cid == id != exclude:
				fn.call(cid)

func _ready() -> void:
	tcp_server.listen(8000)
	Log.info("Server listening on port 8000", Log.SCOPE_SERVER)

	polling_timer.wait_time = 0.2
	polling_timer.autostart = true
	polling_timer.one_shot = false
	polling_timer.timeout.connect(self.poll)
	polling_timer.process_callback = Timer.TIMER_PROCESS_PHYSICS
	self.add_child(polling_timer)
