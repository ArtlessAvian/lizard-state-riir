extends "StateInterface.gd"

var debug_reason = 0
var debug_autoconfirms = 0


func _init(reason: String):
	self.debug_reason = reason


func _poll_input(floor_container: FloorContainer, delta: float):
	var player = floor_container.floor.get_entity_by_id(floor_container.player_id)
	var command = player.get_command_to_confirm()
	if command:
		self.debug_autoconfirms += 1
		floor_container.floor.do_action(command)
		floor_container.emit_signal("floor_dirtied")
		return FloorContainer.ExtraTransitions.NONE

	return FloorContainer.ExtraTransitions.CLEAR
