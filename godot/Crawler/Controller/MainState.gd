class_name MainState
extends "StateInterface.gd"


func _poll_held_input(floor_container: FloorContainer):
	for action in ACTION_TO_DIRECTION:
		if Input.is_action_pressed(action):
			move_player(floor_container, ACTION_TO_DIRECTION[action])
			return FloorContainer.ExtraTransitions.CLEAR

	return FloorContainer.ExtraTransitions.NONE


func _godot_input(floor_container: FloorContainer, event: InputEvent):
	if event is InputEventMouseButton:
		if event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			return goto_mouse(floor_container)

	for action in ACTION_TO_DIRECTION:
		if event.is_action_pressed(action):
			move_player(floor_container, ACTION_TO_DIRECTION[action])
			return FloorContainer.ExtraTransitions.CLEAR

	if event.is_action_pressed("move_wait"):
		var action = PublicActions.get_wait_action()
		var command = action.to_command(floor_container.active_floor, floor_container.player_id)
		if command:
			floor_container.active_floor.do_action(command)
			floor_container.emit_signal("floor_dirtied")
		return FloorContainer.ExtraTransitions.CLEAR

	if event is InputEventKey:
		if event.pressed:
			var player = floor_container.active_floor.get_entity_by_id(floor_container.player_id)
			var actions = player.get_actions()
			if event.physical_keycode == KEY_Q and len(actions) > 0:
				return transition_to_action(floor_container, actions[0])
			if event.physical_keycode == KEY_W and len(actions) > 1:
				return transition_to_action(floor_container, actions[1])
			if event.physical_keycode == KEY_E and len(actions) > 2:
				return transition_to_action(floor_container, actions[2])
			if event.physical_keycode == KEY_R and len(actions) > 3:
				return transition_to_action(floor_container, actions[3])

	return FloorContainer.ExtraTransitions.NONE


func move_player(floor_container: FloorContainer, dir: Vector2i):
	var action: DirectionAction = PublicActions.get_step_macro_action()
	var command = action.to_command(floor_container.active_floor, floor_container.player_id, dir)
	if command:
		floor_container.active_floor.do_action(command)
		floor_container.emit_signal("floor_dirtied")


func goto_mouse(floor_container: FloorContainer):
	var absolute_position = project_mouse_to_tile(floor_container.get_viewport())
	var player_position = (
		floor_container.active_floor.get_entity_by_id(floor_container.player_id).get_pos()
	)

	if absolute_position == player_position:
		floor_container.active_floor.do_action(
			PublicActions.get_wait_action().to_command(
				floor_container.active_floor, floor_container.player_id
			)
		)
		floor_container.emit_signal("floor_dirtied")
		return FloorContainer.ExtraTransitions.CLEAR
	elif (
		(absolute_position - player_position).x <= 1
		and (absolute_position - player_position).x >= -1
		and (absolute_position - player_position).y <= 1
		and (absolute_position - player_position).y >= -1
	):
		floor_container.active_floor.do_action(
			PublicActions.get_step_macro_action().to_command(
				floor_container.active_floor,
				floor_container.player_id,
				absolute_position - player_position
			)
		)
		floor_container.emit_signal("floor_dirtied")
		return FloorContainer.ExtraTransitions.CLEAR

	floor_container.find_child("Cursor").position = (
		Vector3(absolute_position.x, 0, absolute_position.y) + Vector3.UP * 0.01
	)

	var action: TileAction = PublicActions.get_goto_action()
	var command = action.to_command(
		floor_container.active_floor, floor_container.player_id, absolute_position
	)
	if command:
		floor_container.active_floor.do_action(command)
		floor_container.emit_signal("floor_dirtied")

	return preload("res://Crawler/Controller/AutoconfirmState.gd").new("goto")


func transition_to_action(floor_container: FloorContainer, action: Variant):
	if action is TileAction:
		return preload("res://Crawler/Controller/AimTileActionState.gd").new(
			floor_container, action
		)
	if action is DirectionAction:
		return preload("res://Crawler/Controller/AimDirectionActionState.gd").new(
			floor_container, action
		)
	if action is Action:
		return preload("res://Crawler/Controller/AimUnaimedActionState.gd").new(action)

	assert(false, "Unexpected action type: " + action.get_class())
