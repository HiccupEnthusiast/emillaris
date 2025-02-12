extends Resource 
class_name Message

enum MESSAGE_KIND {
	PLAYER_JOINED,
	HELLO,
}

@export var message_kind: MESSAGE_KIND
@export var payload: Dictionary 

static func hello(info: ClientInfo) -> Message:
	var message = Message.new()
	message.message_kind = MESSAGE_KIND.HELLO
	message.payload = prepare_payload(info)
	return message


static func player_joined(info: ClientInfo) -> Message:
	var message = Message.new()
	message.message_kind = MESSAGE_KIND.PLAYER_JOINED
	message.payload = prepare_payload(info)
	return message

func serialize() -> String:
	return JSON.stringify({
		"kind": MESSAGE_KIND.keys()[message_kind],
		"payload": payload
		})

static func deserialize(json: String) -> Message:
	var res = Message.new()
	var data = JSON.parse_string(json)
	res.message_kind = MESSAGE_KIND.keys().find(data["kind"])
	res.payload = data["payload"]
	return res

static func prepare_payload(_payload: Variant) -> Dictionary:
	var res = {}
	for prop in _payload.get_property_list():
		if prop.usage & PROPERTY_USAGE_SCRIPT_VARIABLE:
			res[prop.name] = _payload[prop.name]
	return res
