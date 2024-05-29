class_name MainState
extends "StateInterface.gd"


func _poll_input(floor_container: FloorContainer, delta: float):
	if Input.is_action_pressed("move_left"):
		move_player(floor_container, Vector2i.LEFT)
		return FloorContainer.ExtraTransitions.NONE
	if Input.is_action_pressed("move_up"):
		move_player(floor_container, Vector2i.UP)
		return FloorContainer.ExtraTransitions.NONE
	if Input.is_action_pressed("move_down"):
		move_player(floor_container, Vector2i.DOWN)
		return FloorContainer.ExtraTransitions.NONE
	if Input.is_action_pressed("move_right"):
		move_player(floor_container, Vector2i.RIGHT)
		return FloorContainer.ExtraTransitions.NONE
	if Input.is_action_pressed("move_upleft"):
		move_player(floor_container, Vector2i.UP + Vector2i.LEFT)
		return FloorContainer.ExtraTransitions.NONE
	if Input.is_action_pressed("move_upright"):
		move_player(floor_container, Vector2i.UP + Vector2i.RIGHT)
		return FloorContainer.ExtraTransitions.NONE
	if Input.is_action_pressed("move_downleft"):
		move_player(floor_container, Vector2i.DOWN + Vector2i.LEFT)
		return FloorContainer.ExtraTransitions.NONE
	if Input.is_action_pressed("move_downright"):
		move_player(floor_container, Vector2i.DOWN + Vector2i.RIGHT)
		return FloorContainer.ExtraTransitions.NONE
	if Input.is_action_pressed("move_wait"):
		move_player(floor_container, Vector2i.ZERO)
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
