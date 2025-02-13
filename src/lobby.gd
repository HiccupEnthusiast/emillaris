extends Node

func on_client_connected(c_info: Dictionary) -> void:
	var icon = null
	if c_info.is_host:
		icon = load("res://icon.svg")
	$Players/ItemList.add_item("%s (%s)" % [c_info.name, c_info.id], icon, false)
