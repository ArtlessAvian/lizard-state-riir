extends Node

signal done_animating

var id_to_node: Dictionary
# TODO: Consider pop_front instead? So like a deque.
var event_index = 0
var desynced_from_floor = false

var test_event_delay = 0
var test_tweens = Dictionary()  # Abuse of dictionary. Treat as Array[Pair<Tween, Variant>]

var test_visions: Dictionary  # of EntityIds to their most recent vision.


func _ready():
	desynced_from_floor = true

	($WorldSkew/Map as GridMap).clear()
	($WorldSkew/MapHistory as GridMap).clear()


func _process_floor(delta, floor: Floor):
	if event_index < len(floor.log):
		desynced_from_floor = true
		clear_queue(delta, floor)

	$Control/Label.text = "hi"
	for key in test_tweens.keys():
		$Control/Label.text += (
			"\n" + str(key) + ": " + str(test_tweens[key]) + " " + str(key.is_running())
		)

	for tween in test_tweens.keys().filter(func(t): return !t.is_running()):
		test_tweens.erase(tween)

	if desynced_from_floor and event_index == len(floor.log):
		if test_event_delay > 0:
			test_event_delay -= delta
			return
		if test_tweens.keys().any(func(t): return t.is_running()):
			return
		test_tweens.clear()

		desynced_from_floor = false
		sync_with_engine(floor)
		emit_signal("done_animating")  # repoll input for smooth movement.


func clear_queue(delta, floor: Floor):
	if test_event_delay > 0:
		test_event_delay -= delta
		return
	while event_index < len(floor.log):
		var event = floor.log[event_index]

		if event is MoveEvent:
			if test_tweens.values().any(func(x): return x != "Move"):
				return

			var subject = id_to_node[event.subject]
			var tile = Vector3(event.tile.x, 0, event.tile.y)
			if event.tile != subject.last_known_position:
				subject.get_node("DiscardBasis/Sprite3D").look_dir = tile - subject.position

			if event.tile != subject.last_known_position:
				var tween = subject.create_tween()
				tween.tween_property(subject, "position", tile, 10 / 60.0)
				test_tweens[tween] = "Move"

			subject.last_known_position = event.tile
			event_index += 1

		elif event is StartAttackEvent:
			if test_tweens.keys().any(func(t): return t.is_running()):
				return

			# TODO: Replace with actual animation player.
			var subject = id_to_node[event.subject]
			var target = Vector3(event.tile.x, 0, event.tile.y)
			subject.get_node("DiscardBasis/Sprite3D").look_dir = target - subject.position

			var sprite = subject.get_node("DiscardBasis/Sprite3D")
			sprite.look_dir_offset = 0

			var tween = sprite.create_tween()
			tween.tween_property(sprite, "look_dir_offset", 1, 8 / 60.0)
			tween.parallel().tween_property(
				sprite, "position", sprite.position.lerp(target - subject.position, 0.5), 4 / 60.0
			)
			tween.tween_property(sprite, "position", sprite.position, 4 / 60.0)

			test_tweens[tween] = "Start Attack"

			test_event_delay += 4 / 60.0
			event_index += 1

		elif event is AttackHitEvent:
			var target = id_to_node[event.target]
			target.get_node("DiscardBasis/DamagePopup").popup(-1)

			var subject = id_to_node[event.subject]
			target.get_node("DiscardBasis/Sprite3D").look_dir = subject.position - target.position

			event_index += 1

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

		elif event is KnockbackEvent:
			var subject = id_to_node[event.subject]
			var tile = Vector3(event.tile.x, 0, event.tile.y)

			var tween = subject.create_tween()
			(
				tween
				. tween_property(subject, "position", tile, 20 / 60.0)
				. set_trans(Tween.TRANS_EXPO)
				. set_ease(Tween.EASE_OUT)
			)
			test_tweens[tween] = "Knockback"

			subject.last_known_position = event.tile

			event_index += 1

		else:
			printerr("Unknown Event! ", event)
			event_index += 1


func sync_with_engine(floor):
	for id in floor.get_entity_ids():
		if not id in id_to_node:
			var dup = %Entity.duplicate()
			id_to_node[id] = dup
			dup.name = id.petname
			%Entity.add_sibling(dup)
		var entity = floor.get_entity_by_id(id)
		id_to_node[id].sync_with(floor.get_entity_by_id(id))

	#print(floor.get_entity_by_id(player_id).get_actions())


func _on_floor_container_floor_dirtied():
	desynced_from_floor = true
