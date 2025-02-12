extends Node

var current_scene = null

func _ready() -> void:
	var root = get_tree().root
	current_scene = root.get_child(-1)

func goto_scene(path: String, callback: Callable = func (_s: Variant) -> void: pass) -> void:
	__deferred_goto_scene.call_deferred(path, callback)

func __deferred_goto_scene(path: String, callback: Callable = func (_s: Variant) -> void: pass) -> void:
	current_scene.free()

	var s = ResourceLoader.load(path)
	current_scene = s.instantiate()
	get_tree().root.add_child(current_scene)
	get_tree().current_scene = current_scene
	callback.call(current_scene)
