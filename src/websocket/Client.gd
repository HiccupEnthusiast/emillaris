extends Node
class_name Client

var info := ClientInfo.new()
var websocket := WebSocketPeer.new()
var polling_timer := Timer.new()

signal player_joined(info: ClientInfo)

func send_message(message: String) -> void:
	websocket.send_text(message)

func poll() -> void:
	if websocket.get_ready_state() != WebSocketPeer.STATE_CLOSED:
		websocket.poll()
	while websocket.get_available_packet_count() > 0 and websocket.get_ready_state() == WebSocketPeer.STATE_OPEN:
		var packet = websocket.get_packet().get_string_from_utf8()
		var msg = Message.deserialize(packet)
		if msg.message_kind == Message.MESSAGE_KIND.HELLO:
			info = ClientInfo.new(msg.payload["id"], msg.payload["name"], msg.payload["is_host"])
			player_joined.emit(info)
		if msg.message_kind == Message.MESSAGE_KIND.PLAYER_JOINED:
			var player_info = ClientInfo.new(msg.payload["id"], msg.payload["name"], msg.payload["is_host"])
			player_joined.emit(player_info)
		Log.debug("[%d] Received message: %s" % [info.id, Message.MESSAGE_KIND.keys()[msg.message_kind]], Log.SCOPE_CLIENT)

func _ready() -> void:
	polling_timer.wait_time = 0.2
	polling_timer.autostart = true
	polling_timer.one_shot = false
	polling_timer.timeout.connect(self.poll)
	polling_timer.process_callback = Timer.TIMER_PROCESS_PHYSICS
	self.add_child(polling_timer)

	websocket.supported_protocols = ["emillaris"]
	websocket.connect_to_url("ws://localhost:8000")
