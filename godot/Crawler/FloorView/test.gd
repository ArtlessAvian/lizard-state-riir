extends Node

var player_id: EntityId
var floor: Floor
var id_to_node: Dictionary
# TODO: Consider pop_front instead? So like a deque.
var event_index = 0
var desynced_from_floor = false

var test_event_delay = 0
var test_tweens = []

var test_visions: Dictionary  # of EntityIds to their most recent vision.


func _ready():
	floor = Floor.new()
	# HACK: Temporary.
	floor.set_map($WorldSkew/Map)
	($WorldSkew/Map as GridMap).clear()
	($WorldSkew/MapHistory as GridMap).clear()

	player_id = floor.add_entity_at(Vector2i.ZERO, true)
	id_to_node[player_id] = %Entity

	floor.add_entity_at(Vector2i(-3, 0), false)
	floor.add_entity_at(Vector2i(-3, -1), false)
	desynced_from_floor = true


func _process(delta):
	poll_input(delta)

	floor.take_npc_turn()

	if event_index < len(floor.log):
		desynced_from_floor = true
		clear_queue(delta)

	if desynced_from_floor and event_index == len(floor.log):
		if test_event_delay > 0:
			test_event_delay -= delta
			return
		if test_tweens.any(func(t): return t.is_running()):
			return
		test_tweens.clear()

		desynced_from_floor = false
		sync_with_engine()
		_process(0)  # repoll input for smooth movement.


func clear_queue(delta):
	if test_event_delay > 0:
		test_event_delay -= delta
		return
	if event_index == len(floor.log):
		return

	var event = floor.log[event_index]
	print(event)
	if event is MoveEvent:
		var subject = id_to_node[event.subject]
		var tile = Vector3(event.tile.x, 0, event.tile.y)
		if tile - subject.position != Vector3.ZERO:
			subject.get_node("DiscardBasis/Sprite3D").look_at = tile - subject.position

		var tween = subject.create_tween()
		tween.tween_property(subject, "position", tile, 10 / 60.0)

		test_tweens.push_back(tween)
		event_index += 1
		clear_queue(delta)

	elif event is StartAttackEvent:
		if test_tweens.any(func(t): return t.is_running()):
			return
		test_tweens.clear()

		# TODO: Replace with actual animation player.
		var subject = id_to_node[event.subject]
		var target = Vector3(event.tile.x, 0, event.tile.y)
		subject.get_node("DiscardBasis/Sprite3D").look_at = target - subject.position

		var sprite = subject.get_node("DiscardBasis/Sprite3D")
		var tween = sprite.create_tween()
		tween.tween_property(sprite, "position", sprite.position.lerp(target - subject.position, 0.5), 2 / 60.0)
		tween.tween_property(sprite, "position", sprite.position, 2 / 60.0)

		test_event_delay += 2 / 60.0
		event_index += 1
		clear_queue(delta)

	elif event is AttackHitEvent:
		var target = id_to_node[event.target]
		target.get_node("DiscardBasis/DamagePopup").popup(-1)

		var subject = id_to_node[event.subject]
		target.get_node("DiscardBasis/Sprite3D").look_at = subject.position - target.position

		test_event_delay += 1
		event_index += 1
		clear_queue(delta)

	elif event is SeeMapEvent:
		test_visions[event.subject] = event.vision

		var map = $WorldSkew/Map as GridMap
		map.clear()
		for vision in test_visions.values():
			for pos in vision:
				map.set_cell_item(Vector3i(pos.x, 0, pos.y), 0 if vision[pos] else 1)

		var history = $WorldSkew/MapHistory as GridMap
		for pos in event.vision:
			history.set_cell_item(Vector3i(pos.x, 0, pos.y), 0 if event.vision[pos] else 1)

		event_index += 1
		clear_queue(delta)


func sync_with_engine():
	for id in floor.get_entity_ids():
		if not id in id_to_node:
			var dup = %Entity.duplicate()
			id_to_node[id] = dup
			dup.name = id.petname
			%Entity.add_sibling(dup)
		var entity = floor.get_entity_by_id(id)
		id_to_node[id].position = Vector3(entity.get_pos().x, 0, entity.get_pos().y)


func poll_input(delta):
	if desynced_from_floor:
		return
		
	if Input.is_action_pressed("move_left"):
		move_player(Vector2i.LEFT)
	if Input.is_action_pressed("move_up"):
		move_player(Vector2i.UP)
	if Input.is_action_pressed("move_down"):
		move_player(Vector2i.DOWN)
	if Input.is_action_pressed("move_right"):
		move_player(Vector2i.RIGHT)
	if Input.is_action_pressed("move_upleft"):
		move_player(Vector2i.UP + Vector2i.LEFT)
	if Input.is_action_pressed("move_upright"):
		move_player(Vector2i.UP + Vector2i.RIGHT)
	if Input.is_action_pressed("move_downleft"):
		move_player(Vector2i.DOWN + Vector2i.LEFT)
	if Input.is_action_pressed("move_downright"):
		move_player(Vector2i.DOWN + Vector2i.RIGHT)
	if Input.is_action_pressed("move_wait"):
		move_player(Vector2i.ZERO)
	
	if delta != 0:
		if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
			goto_mouse()

func move_player(dir: Vector2i):
	if desynced_from_floor:
		return

	var player = floor.get_entity_by_id(player_id)
	var action: Action = floor.get_step_macro_action(dir)
	var command = action.to_command(floor, player)
	if command:
		floor.do_action(command)
		desynced_from_floor = true

func goto_mouse():
	if desynced_from_floor:
		return
		
	# HACK: Assumes entire game is on the XZ plane.
	# But this is also kind of expected.
	var mouse = get_viewport().get_mouse_position()
	var origin = get_viewport().get_camera_3d().project_ray_origin(mouse)
	var direction = get_viewport().get_camera_3d().project_ray_normal(mouse)
	
	var projected_xz: Vector3 = origin + (-origin.y / direction.y) * direction
	var rounded = projected_xz.round()
	var absolute_position = Vector2i(rounded.x, rounded.z)
	
	print("absolute position", absolute_position)
	
	var player = floor.get_entity_by_id(player_id)
	var action: Action = floor.get_goto_action(absolute_position)
	var command = action.to_command(floor, player)
	if command:
		floor.do_action(command)
		desynced_from_floor = true
