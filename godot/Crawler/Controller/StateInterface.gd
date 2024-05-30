extends RefCounted

const ACTION_TO_DIRECTION = {
	"move_left": Vector2i.LEFT,
	"move_up": Vector2i.UP,
	"move_down": Vector2i.DOWN,
	"move_right": Vector2i.RIGHT,
	"move_upleft": Vector2i.UP + Vector2i.LEFT,
	"move_upright": Vector2i.UP + Vector2i.RIGHT,
	"move_downleft": Vector2i.DOWN + Vector2i.LEFT,
	"move_downright": Vector2i.DOWN + Vector2i.RIGHT,
	"move_wait": Vector2i.ZERO
}


func _enter(floor_container: FloorContainer):
	pass


func _exit_state(floor_container: FloorContainer):
	pass


# Return a state or Floor.NonStateTransition. Wish there were ADTs.
func _poll_input(floor_container: FloorContainer, delta: float) -> Variant:
	return FloorContainer.ExtraTransitions.NONE


# _unhandled_input, except not explicitly turned on/off per node.
# We want to avoid [Node].set_process_unhandled_input
# so we are sure exactly one node is getting inputs.
func _godot_input(floor_container: FloorContainer, event: InputEvent) -> Variant:
	return FloorContainer.ExtraTransitions.NONE
