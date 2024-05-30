class_name MainState
extends "StateInterface.gd"

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


func _poll_input(floor_container: FloorContainer, delta: float):
	for action in ACTION_TO_DIRECTION:
		if Input.is_action_pressed(action):
			move_player(floor_container, ACTION_TO_DIRECTION[action])
			return FloorContainer.ExtraTransitions.NONE

	var player = floor_container.floor.get_entity_by_id(floor_container.player_id)
	var actions = player.get_actions()
	if Input.is_key_pressed(KEY_Q) and len(actions) > 0:
		return transition_to_action(actions[0])
	if Input.is_key_pressed(KEY_W) and len(actions) > 1:
		return transition_to_action(actions[1])

	return FloorContainer.ExtraTransitions.NONE  # nothing was input


func _godot_input(floor_container: FloorContainer, event: InputEvent):
	if event is InputEventMouseButton:
		if event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			goto_mouse(floor_container)

	return FloorContainer.ExtraTransitions.NONE


func move_player(floor_container: FloorContainer, dir: Vector2i):
	var player = floor_container.floor.get_entity_by_id(floor_container.player_id)
	var action: DirectionAction = floor_container.floor.get_step_macro_action()
	var command = action.to_command(floor_container.floor, player, dir)
	if command:
		floor_container.floor.do_action(command)
		floor_container.emit_signal("floor_dirtied")


func goto_mouse(floor_container: FloorContainer):
	# HACK: Assumes entire game is on the XZ plane.
	# But this is also kind of expected.
	var viewport = floor_container.get_viewport()
	var mouse = viewport.get_mouse_position()
	var origin = viewport.get_camera_3d().project_ray_origin(mouse)
	var direction = viewport.get_camera_3d().project_ray_normal(mouse)

	var projected_xz: Vector3 = origin + (-origin.y / direction.y) * direction
	var rounded = projected_xz.round()
	var absolute_position = Vector2i(rounded.x, rounded.z)

	#$Cursor.position = rounded + Vector3.UP * 0.01

	print("absolute position", absolute_position)

	var player = floor_container.floor.get_entity_by_id(floor_container.player_id)
	var action: TileAction = floor_container.floor.get_goto_action()
	var command = action.to_command(floor_container.floor, player, absolute_position)
	if command:
		floor_container.floor.do_action(command)
		floor_container.emit_signal("floor_dirtied")


func transition_to_action(action: Variant):
	if action is TileAction:
		return preload("res://Crawler/Controller/AimTileActionState.gd").new(action)
	if action is DirectionAction:
		return preload("res://Crawler/Controller/AimDirectionActionState.gd").new(action)
	if action is Action:
		return preload("res://Crawler/Controller/AimUnaimedActionState.gd").new(action)

	assert(false, "Unexpected action type: " + action.get_class())
