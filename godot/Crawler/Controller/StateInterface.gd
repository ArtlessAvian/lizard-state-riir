extends Node


func _enter(floor_container: FloorContainer):
	pass


func _exit_state(floor_container: FloorContainer):
	pass


# Return self or a child to go to that state, or null to return to root
# (unknown by the current node).
func _poll_input(floor_container: FloorContainer, delta: float) -> Node:
	return self


# _unhandled_input, except not explicitly turned on/off per node.
# We want to avoid [Node].set_process_unhandled_input
# so we are sure exactly one node is getting inputs.
func _godot_input(floor_container: FloorContainer, event: InputEvent) -> Node:
	return self
