extends "StateInterface.gd"

var action


func _init(action: InfallibleAction):
	self.action = action


func _godot_input(floor_container: FloorContainer, event: InputEvent) -> Variant:
	if event.is_action_pressed("ui_select"):
		var command = action.to_command(floor_container.active_floor, floor_container.player_id)
		if command:
			floor_container.active_floor.do_action(command)
			floor_container.emit_signal("floor_dirtied")
			return FloorContainer.ExtraTransitions.CLEAR
	if event.is_action_pressed("ui_cancel"):
		return FloorContainer.ExtraTransitions.EXIT

	if event is InputEventMouseButton:
		if event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			var command = action.to_command(floor_container.active_floor, floor_container.player_id)
			if command:
				floor_container.active_floor.do_action(command)
				floor_container.emit_signal("floor_dirtied")
				return FloorContainer.ExtraTransitions.CLEAR

	return FloorContainer.ExtraTransitions.NONE
