extends Node

var id_to_node: Dictionary
# TODO: Consider pop_front instead? So like a deque.
var event_index = 0
var desynced_from_floor = false

var test_event_delay = 0
var test_tweens = []

var test_visions: Dictionary  # of EntityIds to their most recent vision.


func _ready():
	desynced_from_floor = true
	
	($WorldSkew/Map as GridMap).clear()
	($WorldSkew/MapHistory as GridMap).clear()


func _process_floor(delta, floor: Floor):
	if event_index < len(floor.log):
		desynced_from_floor = true
		clear_queue(delta, floor)

	if desynced_from_floor and event_index == len(floor.log):
		if test_event_delay > 0:
			test_event_delay -= delta
			return
		if test_tweens.any(func(t): return t.is_running()):
			return
		test_tweens.clear()

		desynced_from_floor = false
		sync_with_engine(floor)
		_process_floor(0, floor)  # repoll input for smooth movement.


func clear_queue(delta, floor: Floor):
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
		clear_queue(delta, floor)

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
		tween.tween_property(
			sprite, "position", sprite.position.lerp(target - subject.position, 0.5), 2 / 60.0
		)
		tween.tween_property(sprite, "position", sprite.position, 2 / 60.0)

		test_event_delay += 2 / 60.0
		event_index += 1
		clear_queue(delta, floor)

	elif event is AttackHitEvent:
		var target = id_to_node[event.target]
		target.get_node("DiscardBasis/DamagePopup").popup(-1)

		var subject = id_to_node[event.subject]
		target.get_node("DiscardBasis/Sprite3D").look_at = subject.position - target.position

		test_event_delay += 1
		event_index += 1
		clear_queue(delta, floor)

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
		clear_queue(delta, floor)


func sync_with_engine(floor):
	for id in floor.get_entity_ids():
		if not id in id_to_node:
			var dup = %Entity.duplicate()
			id_to_node[id] = dup
			dup.name = id.petname
			%Entity.add_sibling(dup)
		var entity = floor.get_entity_by_id(id)
		id_to_node[id].position = Vector3(entity.get_pos().x, 0, entity.get_pos().y)
		id_to_node[id].find_child("Debug").text = entity.get_debug()

	#print(floor.get_entity_by_id(player_id).get_actions())


func _on_floor_container_floor_dirtied():
	desynced_from_floor = true
