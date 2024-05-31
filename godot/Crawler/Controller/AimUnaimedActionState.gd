extends "StateInterface.gd"

var action


func _init(action: Action):
	self.action = action


func _godot_input(floor_container: FloorContainer, event: InputEvent) -> Variant:
	if event.is_action_pressed("ui_select"):
		var player = floor_container.floor.get_entity_by_id(floor_container.player_id)
		var command = action.to_command(floor_container.floor, player)
		if command:
			floor_container.floor.do_action(command)
			floor_container.emit_signal("floor_dirtied")
			return FloorContainer.ExtraTransitions.CLEAR

	return FloorContainer.ExtraTransitions.NONE
