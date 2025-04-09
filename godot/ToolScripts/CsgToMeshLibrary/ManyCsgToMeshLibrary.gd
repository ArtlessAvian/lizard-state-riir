@tool
extends Node

@export_tool_button("Bake All") var bake_button = bake_all


func bake_all():
	for child in get_children():
		child.bake()
