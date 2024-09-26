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


func _process_floor(delta, floor: ActiveFloor):
	if test_event_delay > 0:
		test_event_delay -= delta
		return

	if event_index < len(floor.log):
		desynced_from_floor = true
		clear_queue(delta, floor)

	for tween in test_tweens.keys().filter(func(t): return !t.is_valid()):
		test_tweens.erase(tween)

	if (
		desynced_from_floor
		and event_index == len(floor.log)
		and test_event_delay <= 0
		and test_tweens.is_empty()
	):
		desynced_from_floor = false
		sync_with_engine(floor)
		emit_signal("done_animating")  # repoll input for smooth movement.

	$Control/Label.text = "hi"
	for key in test_tweens.keys():
		$Control/Label.text += (
			"\n" + str(key) + ": " + str(test_tweens[key]) + " " + str(key.is_valid())
		)


func clear_queue(delta, floor: ActiveFloor):
	while event_index < len(floor.log):
		var event = floor.log[event_index]

		if event is MoveEvent:
			if test_tweens.values().any(func(x): return x != "Move"):
				return

			var subject = id_to_node[event.subject]
			var tile = Vector3(event.tile.x, 0, event.tile.y)
			if event.tile != subject.last_known_position:
				subject.basis = Basis.looking_at(tile - subject.position, Vector3.UP, true)

			if event.tile != subject.last_known_position:
				var tween = subject.create_tween()
				# HACK: Tweens do work after process, *after* the camera has tracked the player
				# position, so the player moves after the camera has moved.
				# call_deferred does not help much?
				# This is not a good solution because movement is now locked to 60 FPS.
				# We could instead reparent the camera but it is a bit awkward.
				tween.set_process_mode(Tween.TWEEN_PROCESS_PHYSICS)
				tween.tween_property(subject, "position", tile, 10 / 60.0)
				test_tweens[tween] = "Move"
				# HACK: For smoothness at very low fps. The best thing to do would be to pass
				# a residual delta from done_animating, and pass that back into floor_sync.
				# Then use delta here.
				# tween.custom_step(1.0 / max(Engine.get_frames_per_second(), 144))

			subject.last_known_position = event.tile
			event_index += 1

		elif event is StartAttackEvent:
			if test_tweens.keys().any(func(t): return t.is_valid()):
				return

			# TODO: Replace with actual animation player.
			var subject = id_to_node[event.subject]
			var target = Vector3(event.tile.x, 0, event.tile.y)
			subject.basis = Basis.looking_at(target - subject.position, Vector3.UP, true)

			var animation = subject.get_node("AnimationPlayer") as AnimationPlayer
			animation.play("Entity/Attack")

			test_event_delay += 4 / 60.0
			event_index += 1

		elif event is AttackHitEvent:
			var target = id_to_node[event.target]
			target.get_node("DiscardBasis/DamagePopup").popup(-1)

			var subject = id_to_node[event.subject]
			target.basis = Basis.looking_at(subject.position - target.position, Vector3.UP, true)

			var animation = target.get_node("AnimationPlayer") as AnimationPlayer
			animation.play("Entity/Hurt")

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

		elif event is KnockdownEvent:
			var subject = id_to_node[event.subject]
			var animation = subject.get_node("AnimationPlayer") as AnimationPlayer
			animation.play(&"Entity/KnockedDown")

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
		id_to_node[id].sync_with(floor.get_entity_by_id(id))

	#print(floor.get_entity_by_id(player_id).get_actions())


func _on_floor_container_floor_dirtied():
	desynced_from_floor = true
