extends Node

enum LOG_LEVEL {
	DEBUG,
	INFO,
	WARN,
	ERROR
}

const SCOPE_SERVER = "Server"
const SCOPE_CLIENT = "Client"

var pretty_print_activated := true

func debug(message: String, scope: String) -> void:
	msg(LOG_LEVEL.DEBUG, message, scope)

func info(message: String, scope: String) -> void:
	msg(LOG_LEVEL.INFO, message, scope)

func warn(message: String, scope: String) -> void:
	msg(LOG_LEVEL.WARN, message, scope)

func error(message: String, scope: String) -> void:
	msg(LOG_LEVEL.ERROR, message, scope)

func msg(level: LOG_LEVEL, message: String, scope: String) -> void:
	var lvl: String
	if pretty_print_activated:
		match level:
			LOG_LEVEL.DEBUG: lvl = "[color=green]DEBUG[/color]"
			LOG_LEVEL.INFO: lvl = "[color=blue]INFO[/color]"
			LOG_LEVEL.WARN: lvl = "[color=yellow]WARN[/color]"
			LOG_LEVEL.ERROR: lvl = "[color=red]ERROR[/color]"
	else:
		match level:
			LOG_LEVEL.DEBUG: lvl = "DEBUG"
			LOG_LEVEL.INFO: lvl = "INFO"
			LOG_LEVEL.WARN: lvl = "WARN"
			LOG_LEVEL.ERROR: lvl = "ERROR"

	print_rich("%s [%s] [%s] %s" % [lvl, Time.get_datetime_string_from_system(true), scope, message])
