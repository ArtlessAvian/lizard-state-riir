extends Node

signal done_animating

var id_to_node: Dictionary
# TODO: Consider pop_front instead? So like a deque.
var event_index = 0
var desynced_from_floor = false

var test_event_delay = 0

var test_visions: Dictionary  # of EntityIds to their most recent vision.

var stop_processing: bool = false


func _ready():
	desynced_from_floor = true

	($WorldSkew/Map as GridMap).clear()
	($WorldSkew/MapHistory as GridMap).clear()


func _process_floor(delta, floor: ActiveFloor):
	if stop_processing:
		return

	if test_event_delay > 0:
		test_event_delay -= delta
		return

	if event_index < len(floor.log):
		desynced_from_floor = true
		clear_queue(delta, floor)

	if (
		desynced_from_floor
		and event_index == len(floor.log)
		and test_event_delay <= 0
		and id_to_node.values().all(func(x): return not x.is_animating())
	):
		desynced_from_floor = false
		sync_with_engine(floor)
		emit_signal("done_animating")  # repoll input for smooth movement.

	$Control/Label.text = "hi"
	for id in id_to_node:
		var node = id_to_node[id]
		if node.is_animating():
			$Control/Label.text += (
				"\n"
				+ str(id)
				+ ": "
				+ str(node.movement_tween)
				+ " "
				+ str(node.get_node("AnimationPlayer").current_animation)
			)


func clear_queue(delta, floor: ActiveFloor):
	while event_index < len(floor.log) and !stop_processing:
		var event = floor.log[event_index]

		if event is MoveEvent:
			var subject = id_to_node[event.subject]
			if subject.is_animating():
				return

			var tile = Vector3(event.tile.x, 0, event.tile.y)
			if event.tile != subject.last_known_position:
				subject.basis = Basis.looking_at(tile - subject.position, Vector3.UP, true)

			if event.tile != subject.last_known_position:
				var tween = subject.create_tween()
				tween.tween_property(subject, "position", tile, 10 / 60.0)
				subject.movement_tween = tween
				subject.last_known_position = event.tile

			event_index += 1

		elif event is PrepareAttackEvent:
			if id_to_node.values().any(func(x): return x.is_important_animating()):
				return

			# TODO: Replace with actual animation player.
			var subject = id_to_node[event.subject]
			var target = Vector3(event.tile.x, 0, event.tile.y)
			subject.basis = Basis.looking_at(target - subject.position, Vector3.UP, true)

			var animation = subject.get_node("AnimationPlayer") as AnimationPlayer
			animation.play("Entity/StateCommitted")

			test_event_delay += 4 / 60.0
			event_index += 1

		elif event is StartAttackEvent:
			if id_to_node.values().any(func(x): return x.is_animating()):
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

		elif event is JuggleHitEvent:
			var target = id_to_node[event.target]

			var animation = target.get_node("AnimationPlayer") as AnimationPlayer
			animation.play("Entity/GetJuggled")

			event_index += 1

		elif event is JuggleLimitEvent:
			var target = id_to_node[event.target]

			var animation = target.get_node("AnimationPlayer") as AnimationPlayer
			animation.play("Entity/GetJuggleLimited")

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
			if subject.is_important_animating():
				return

			var tile = Vector3(event.tile.x, 0, event.tile.y)

			var tween = subject.create_tween()
			(
				tween
				. tween_property(subject, "position", tile, 20 / 60.0)
				. set_trans(Tween.TRANS_EXPO)
				. set_ease(Tween.EASE_OUT)
			)
			subject.movement_tween = tween
			subject.last_known_position = event.tile

			event_index += 1

		elif event is KnockdownEvent:
			var subject = id_to_node[event.subject]
			var animation = subject.get_node("AnimationPlayer") as AnimationPlayer
			animation.play(&"Entity/KnockedDown")

			event_index += 1

		elif event is WakeupEvent:
			var subject = id_to_node[event.subject]
			if subject.is_important_animating():
				return
			var animation = subject.get_node("AnimationPlayer") as AnimationPlayer
			animation.play(&"Entity/Wakeup")

			event_index += 1

		elif event is GetDownedEvent:
			var subject = id_to_node[event.subject]
			if subject.is_important_animating():
				return
			var animation = subject.get_node("AnimationPlayer") as AnimationPlayer
			animation.play(&"Entity/StateDowned")

			event_index += 1

		elif event is MissionFailedEvent:
			if id_to_node.values().any(func(x): return x.is_animating()):
				return

			stop_processing = true

			var subject = id_to_node[event.subject]
			subject.get_node("AnimationPlayer").play("RESET")

			var tween: Tween = subject.create_tween()
			tween.tween_interval(1)
			var the_position = subject.position
			for entity in event.downed_party.map(func(x): return id_to_node[x]):
				tween.tween_property(
					subject,
					"basis",
					Basis.looking_at(entity.position - the_position, Vector3.UP, true),
					0
				)
				(
					tween
					. tween_property(subject, "position", entity.position, 20 / 60.0)
					. set_trans(Tween.TRANS_EXPO)
					. set_ease(Tween.EASE_OUT)
				)
				tween.tween_interval(0.1)
				tween.tween_property(entity, "visible", false, 0)
				tween.tween_interval(0.1)

				the_position = entity.position

			tween.tween_property(subject.get_node("AnimatedSprite3D"), "position:y", 300, 10)

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
