extends Resource
class_name ClientInfo 

@export var id: int
@export var name: String
@export var is_host: bool
@export var is_ready: bool

func _init(_id = 2,_name = "N/A", _is_host = false, _ready = false):
	self.id = _id
	self.name = _name 
	self.is_host = _is_host 
	self.is_ready = false
