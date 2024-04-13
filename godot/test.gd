extends Node

var player_id: EntityId
var floor: Floor
var id_to_node: Dictionary
# TODO: Consider pop_front instead? So like a deque.
var event_index = 0
var desynced_from_floor = false

var test_event_delay = 0


func _ready():
	floor = Floor.new()

	player_id = floor.add_entity_at(Vector2i.ZERO)
	id_to_node[player_id] = %Entity

	floor.add_entity_at(Vector2i(-5, 1))
	floor.add_entity_at(Vector2i(-5, 0))
	desynced_from_floor = true


func _process(delta):
	poll_input()

	floor.take_npc_turn()

	if event_index < len(floor.log):
		desynced_from_floor = true
		clear_queue(delta)

	if desynced_from_floor and event_index == len(floor.log):
		desynced_from_floor = false
		sync_with_engine()


func clear_queue(delta):
	if test_event_delay > 0:
		test_event_delay -= delta
		return

	if event_index + 1 < len(floor.log):
		test_event_delay += 0.5

	var event = floor.log[event_index]
	print(event)
	if event is MoveEvent:
		prints(event.subject, event.tile)
		id_to_node[event.subject].position = Vector3(event.tile.x, 0, event.tile.y)

	event_index += 1


func sync_with_engine():
	for id in floor.get_entity_ids():
		if not id in id_to_node:
			var dup = %Entity.duplicate()
			id_to_node[id] = dup
			dup.name = id.petname
			%Entity.get_parent().add_child(dup)
		var entity = floor.get_entity_by_id(id)
		id_to_node[id].position = Vector3(entity.get_pos().x, 0, entity.get_pos().y)


func poll_input():
	if Input.is_action_just_pressed("move_left"):
		move_player(Vector2i.LEFT)
	if Input.is_action_just_pressed("move_up"):
		move_player(Vector2i.UP)
	if Input.is_action_just_pressed("move_down"):
		move_player(Vector2i.DOWN)
	if Input.is_action_just_pressed("move_right"):
		move_player(Vector2i.RIGHT)
	if Input.is_action_just_pressed("move_upleft"):
		move_player(Vector2i.UP + Vector2i.LEFT)
	if Input.is_action_just_pressed("move_upright"):
		move_player(Vector2i.UP + Vector2i.RIGHT)
	if Input.is_action_just_pressed("move_downleft"):
		move_player(Vector2i.DOWN + Vector2i.LEFT)
	if Input.is_action_just_pressed("move_downright"):
		move_player(Vector2i.DOWN + Vector2i.RIGHT)


func move_player(dir: Vector2i):
	if desynced_from_floor:
		return

	var player = floor.get_entity_by_id(player_id)
	var action: Action = floor.get_step_macro_action(dir)
	var command = action.to_command(floor, player)
	if command:
		floor.do_action(command)

	# just for fun
	%Entity/DiscardBasis/Sprite3D.look_at = Vector3(dir.x, 0, dir.y)
