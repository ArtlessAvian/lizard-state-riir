class_name FloorContainer
extends Node3D

signal floor_dirtied

var floor: Floor
var player_id: EntityId

var input_state: Node = null  # set to not null in _ready() :P


# Called when the node enters the scene tree for the first time.
func _ready():
	floor = Floor.new()
	# HACK: Temporary.
	floor.set_map_2d($Map)

	player_id = floor.add_entity_at(Vector2i.ZERO, true)
	$Floor.id_to_node[player_id] = find_child("Entity")

	floor.add_entity_at(Vector2i(3, 0), false)
	#floor.add_entity_at(Vector2i(-3, -1), false)

	input_state = $InputStates/Main
	input_state._enter(self)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	if not $Floor.desynced_from_floor:
		input_state._poll_input(self, delta)

	floor.take_npc_turn()
	$Floor._process_floor(delta, floor)


func _unhandled_input(event):
	if not $Floor.desynced_from_floor:
		input_state._godot_input(self, event)
