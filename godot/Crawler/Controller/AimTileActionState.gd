extends "StateInterface.gd"

var action
var absolute_position


func _init(floor_container: FloorContainer, action: TileAction):
	var player = floor_container.floor.get_entity_by_id(floor_container.player_id)
	self.absolute_position = player.get_pos()

	self.action = action

	floor_container.find_child("Cursor").position = Vector3(
		self.absolute_position.x, 0.01, self.absolute_position.y
	)


func _poll_input(floor_container: FloorContainer, delta: float) -> Variant:
	poll_without_transition(floor_container, delta)

	if Input.is_action_just_pressed("ui_select"):
		var player = floor_container.floor.get_entity_by_id(floor_container.player_id)
		var command = action.to_command(floor_container.floor, player, absolute_position)
		if command:
			floor_container.floor.do_action(command)
			floor_container.emit_signal("floor_dirtied")
			return FloorContainer.ExtraTransitions.CLEAR

	return FloorContainer.ExtraTransitions.NONE


func poll_without_transition(floor_container: FloorContainer, delta: float):
	for action in ACTION_TO_DIRECTION:
		if Input.is_action_pressed(action):
			self.absolute_position += ACTION_TO_DIRECTION[action]

			floor_container.find_child("Cursor").position = Vector3(
				self.absolute_position.x, 0.01, self.absolute_position.y
			)
