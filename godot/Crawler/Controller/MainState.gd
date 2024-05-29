extends "StateInterface.gd"


func _poll_input(floor_container: FloorContainer, delta: float):
	if Input.is_action_pressed("move_left"):
		move_player(floor_container, Vector2i.LEFT)
		return self
	if Input.is_action_pressed("move_up"):
		move_player(floor_container, Vector2i.UP)
		return self
	if Input.is_action_pressed("move_down"):
		move_player(floor_container, Vector2i.DOWN)
		return self
	if Input.is_action_pressed("move_right"):
		move_player(floor_container, Vector2i.RIGHT)
		return self
	if Input.is_action_pressed("move_upleft"):
		move_player(floor_container, Vector2i.UP + Vector2i.LEFT)
		return self
	if Input.is_action_pressed("move_upright"):
		move_player(floor_container, Vector2i.UP + Vector2i.RIGHT)
		return self
	if Input.is_action_pressed("move_downleft"):
		move_player(floor_container, Vector2i.DOWN + Vector2i.LEFT)
		return self
	if Input.is_action_pressed("move_downright"):
		move_player(floor_container, Vector2i.DOWN + Vector2i.RIGHT)
		return self
	if Input.is_action_pressed("move_wait"):
		move_player(floor_container, Vector2i.ZERO)
		return self

	if Input.is_key_pressed(KEY_Q):
		var player = floor_container.floor.get_entity_by_id(floor_container.player_id)
		var action = player.get_actions()[0]
		var command = action.to_command(floor_container.floor, player, Vector2i(0, 1))
		if command:
			floor_container.floor.do_action(command)
			floor_container.emit_signal("floor_dirtied")
		return self
	if Input.is_key_pressed(KEY_W):
		var player = floor_container.floor.get_entity_by_id(floor_container.player_id)
		var action = player.get_actions()[1]
		var command = action.to_command(floor_container.floor, player)
		if command:
			floor_container.floor.do_action(command)
			floor_container.emit_signal("floor_dirtied")
		return self

	return self  # nothing was input


func _godot_input(floor_container: FloorContainer, event: InputEvent) -> Node:
	if event is InputEventMouseButton:
		if event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			goto_mouse(floor_container)

	return self


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
	var mouse = get_viewport().get_mouse_position()
	var origin = get_viewport().get_camera_3d().project_ray_origin(mouse)
	var direction = get_viewport().get_camera_3d().project_ray_normal(mouse)

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
