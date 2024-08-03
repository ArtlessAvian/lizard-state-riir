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


func _godot_input(floor_container: FloorContainer, event: InputEvent) -> Variant:
	godot_input_without_transition(floor_container, event)

	if event.is_action_pressed("ui_select"):
		var command = action.to_command(
			floor_container.floor, floor_container.player_id, absolute_position
		)
		if command:
			floor_container.floor.do_action(command)
			floor_container.emit_signal("floor_dirtied")
			return FloorContainer.ExtraTransitions.CLEAR
	if event.is_action_pressed("ui_cancel"):
		return FloorContainer.ExtraTransitions.EXIT
		
	if event is InputEventMouseButton:
		if event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			var command = action.to_command(
				floor_container.floor,
				floor_container.player_id,
				project_mouse_to_tile(floor_container.get_viewport())
			)
			if command:
				floor_container.floor.do_action(command)
				floor_container.emit_signal("floor_dirtied")
				return FloorContainer.ExtraTransitions.CLEAR

	return FloorContainer.ExtraTransitions.NONE


func godot_input_without_transition(floor_container: FloorContainer, event: InputEvent):
	for action in ACTION_TO_DIRECTION:
		if event.is_action_pressed(action):
			self.absolute_position += ACTION_TO_DIRECTION[action]

			floor_container.find_child("Cursor").position = Vector3(
				self.absolute_position.x, 0.01, self.absolute_position.y
			)

	if event is InputEventMouseMotion:
		var projected = project_mouse_to_tile(floor_container.get_viewport())
		floor_container.find_child("Cursor").position = Vector3(projected.x, 0.01, projected.y)
