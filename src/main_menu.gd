extends Node

func _on_host() -> void:
	Global.goto_scene("res://scenes/lobby.tscn", func (scene: Node):
		var server = Server.new()
		var client = Client.new()
		client.connection_established.connect(scene.on_client_connected)
		client.client_joined.connect(scene.on_client_connected)
		scene.add_child(server)
		scene.add_child(client)
	)

func _on_join() -> void:
	Global.goto_scene("res://scenes/lobby.tscn", func (scene: Node):
		var client = Client.new()
		client.connection_established.connect(scene.on_client_connected)
		client.client_joined.connect(scene.on_client_connected)
		scene.add_child(client)
	)
