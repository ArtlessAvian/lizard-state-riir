extends Node

var ci_version: String = ""


func _ready() -> void:
	ci_version = FileAccess.get_file_as_string("res://ci-version.txt")
