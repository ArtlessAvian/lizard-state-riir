class_name MainState
extends "StateInterface.gd"


func _godot_input(floor_container: FloorContainer, event: InputEvent):
	if event is InputEventMouseButton:
		if event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			goto_mouse(floor_container)

	for action in ACTION_TO_DIRECTION:
		if event.is_action_pressed(action):
			move_player(floor_container, ACTION_TO_DIRECTION[action])
			return FloorContainer.ExtraTransitions.NONE

	if event is InputEventKey:
		if event.pressed:
			var player = floor_container.floor.get_entity_by_id(floor_container.player_id)
			var actions = player.get_actions()
			if event.physical_keycode == KEY_Q and len(actions) > 0:
				return transition_to_action(floor_container, actions[0])
			if event.physical_keycode == KEY_W and len(actions) > 1:
				return transition_to_action(floor_container, actions[1])

	return FloorContainer.ExtraTransitions.NONE


func move_player(floor_container: FloorContainer, dir: Vector2i):
	var player = floor_container.floor.get_entity_by_id(floor_container.player_id)
	var action: DirectionAction = floor_container.floor.get_step_macro_action()
	var command = action.to_command(floor_container.floor, player, dir)
	if command:
		floor_container.floor.do_action(command)
		floor_container.emit_signal("floor_dirtied")


func goto_mouse(floor_container: FloorContainer):
	var absolute_position = project_mouse_to_tile(floor_container.get_viewport())

	floor_container.find_child("Cursor").position = (
		Vector3(absolute_position.x, 0, absolute_position.y) + Vector3.UP * 0.01
	)

	var player = floor_container.floor.get_entity_by_id(floor_container.player_id)
	var action: TileAction = floor_container.floor.get_goto_action()
	var command = action.to_command(floor_container.floor, player, absolute_position)
	if command:
		floor_container.floor.do_action(command)
		floor_container.emit_signal("floor_dirtied")


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
