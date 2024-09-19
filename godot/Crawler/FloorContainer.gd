class_name FloorContainer
extends Node3D

signal floor_dirtied

enum ExtraTransitions { NONE, EXIT, CLEAR }

@onready var ROOT_STATE = MainState.new()

var active_floor: ActiveFloor
var player_id: EntityId

var input_state_stack: Array = []

var debug_frame_times: Array = [0, 0, 0, 0, 0]


# Called when the node enters the scene tree for the first time.
func _ready():
	active_floor = ActiveFloor.new()
	# HACK: Temporary.
	active_floor.set_map_2d($Map)

	player_id = active_floor.add_entity_at(Vector2i.ZERO, true, true)
	$FloorView.id_to_node[player_id] = find_child("Entity")

	active_floor.add_entity_at(Vector2i(3, 0), false, true)

	# Entities from old game.
	active_floor.add_entity_at(Vector2i(21, 10), false, false)  # enemy
	active_floor.add_entity_at(Vector2i(8, 34), false, false)  # enemy3
	# out of bounds enemy omitted.
	active_floor.add_entity_at(Vector2i(-11, -5), false, false)  # enemy
	active_floor.add_entity_at(Vector2i(35, 4), false, false)  # enemy
	active_floor.add_entity_at(Vector2i(17, -29), false, false)  # enemy2
	active_floor.add_entity_at(Vector2i(-18, -12), false, false)  # enemy2
	active_floor.add_entity_at(Vector2i(23, -17), false, false)  # enemy2
	active_floor.add_entity_at(Vector2i(9, -25), false, false)  # enemy2
	active_floor.add_entity_at(Vector2i(16, 16), false, false)  # enemy2


func get_current_state() -> RefCounted:
	if not input_state_stack.is_empty():
		return input_state_stack.back()
	return ROOT_STATE


func do_transition(transition: Variant):
	if transition is ExtraTransitions:
		match transition:
			ExtraTransitions.NONE:
				pass
			ExtraTransitions.EXIT:
				input_state_stack.pop_back()
			ExtraTransitions.CLEAR:
				input_state_stack.clear()
	else:
		input_state_stack.push_back(transition)


func run_engine(frame_start: int):
	while true:
		if not active_floor.take_npc_turn():
			break
		if Time.get_ticks_usec() - frame_start > 1000000.0 / 30:
			push_error("Engine taking too long!")
			break


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	var frame_start = Time.get_ticks_usec()

	if not $FloorView.desynced_from_floor:
		var transition = get_current_state()._poll_input(self, delta)
		do_transition(transition)

	run_engine(frame_start)
	# May emit done_animating.
	$FloorView._process_floor(delta, active_floor)

	var frame_time = Time.get_ticks_usec() - frame_start
	if frame_time > debug_frame_times[0]:
		debug_frame_times.insert(debug_frame_times.bsearch(frame_time), frame_time)
		debug_frame_times.pop_front()

	var debug_state_stack = []
	for state in input_state_stack:
		debug_state_stack.push_back(state.get_script().get_path())

	$DEBUG.text = ""
	$DEBUG.text += "worst engine times (us): " + str(debug_frame_times) + "\n"
	$DEBUG.text += "turn count: " + str(active_floor.get_time()) + "\n"
	$DEBUG.text += "input stack: " + "\n".join(debug_state_stack) + "\n"


func _unhandled_input(event):
	if not $FloorView.desynced_from_floor:
		var transition = get_current_state()._godot_input(self, event)
		do_transition(transition)


func _on_floor_view_done_animating() -> void:
	var transition = get_current_state()._poll_held_input(self)
	do_transition(transition)

	run_engine(Time.get_ticks_usec())
	# May emit done_animating. Recurring is possible, but not infinite.
	$FloorView._process_floor(0, active_floor)
