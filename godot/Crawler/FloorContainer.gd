class_name FloorContainer
extends Node3D

signal floor_dirtied

enum ExtraTransitions { NONE, EXIT, CLEAR }

@onready var ROOT_STATE = MainState.new()

var floor: Floor
var player_id: EntityId

var input_state_stack: Array = []


# Called when the node enters the scene tree for the first time.
func _ready():
	floor = Floor.new()
	# HACK: Temporary.
	floor.set_map_2d($Map)

	player_id = floor.add_entity_at(Vector2i.ZERO, true)
	$FloorView.id_to_node[player_id] = find_child("Entity")

	floor.add_entity_at(Vector2i(3, 0), false)
	#floor.add_entity_at(Vector2i(-3, -1), false)


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


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	if not $FloorView.desynced_from_floor:
		var transition = get_current_state()._poll_input(self, delta)
		do_transition(transition)

	floor.take_npc_turn()
	$FloorView._process_floor(delta, floor)

	var debug_state_str = ""
	for state in input_state_stack:
		debug_state_str += state.get_script().get_path() + "\n"
	$DEBUG.text = debug_state_str + " " + str(floor.get_time())


func _unhandled_input(event):
	if not $FloorView.desynced_from_floor:
		var transition = get_current_state()._godot_input(self, event)
		do_transition(transition)


func _on_floor_view_done_animating() -> void:
	var transition = get_current_state()._poll_held_input(self)
	do_transition(transition)
