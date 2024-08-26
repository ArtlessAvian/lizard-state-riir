extends "StateInterface.gd"

var debug_reason = 0
var debug_autoconfirms = 0


func _init(reason: String):
	self.debug_reason = reason


# TODO: Does not cancel while moving!
# func _godot_input(floor_container: FloorContainer, event: InputEvent) -> Variant:
# 	if event.is_action_pressed("ui_cancel"):
# 		return FloorContainer.ExtraTransitions.EXIT
# 	return FloorContainer.ExtraTransitions.NONE


func _poll_input(floor_container: FloorContainer, delta: float):
# 	# TODO: Does not cancel while moving!
# 	if event.is_action_pressed("ui_cancel"):
# 		return FloorContainer.ExtraTransitions.EXIT

	var player = floor_container.floor.get_entity_by_id(floor_container.player_id)
	var command = player.get_command_to_confirm()
	if command:
		self.debug_autoconfirms += 1
		floor_container.floor.do_action(command)
		floor_container.emit_signal("floor_dirtied")
		return FloorContainer.ExtraTransitions.NONE

	return FloorContainer.ExtraTransitions.CLEAR

func _poll_held_input(floor_container: FloorContainer):
# 	# TODO: Does not cancel while moving!
# 	if event.is_action_pressed("ui_cancel"):
# 		return FloorContainer.ExtraTransitions.EXIT

	var player = floor_container.floor.get_entity_by_id(floor_container.player_id)
	var command = player.get_command_to_confirm()
	if command:
		self.debug_autoconfirms += 1
		floor_container.floor.do_action(command)
		floor_container.emit_signal("floor_dirtied")
		return FloorContainer.ExtraTransitions.NONE

	return FloorContainer.ExtraTransitions.CLEAR
