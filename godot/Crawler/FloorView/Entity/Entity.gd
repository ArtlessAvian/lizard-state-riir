@tool
extends Node3D

# Read-only for tool/editor stuff.
@export var entity_initializer: EntityInitializer

# The actual logic and stuff.
var movement_tween: Tween
var last_known_position: Vector2i


func _ready():
	if Engine.is_editor_hint():
		if entity_initializer is EntityInitializer:
			self.sync_with(entity_initializer.to_snapshot())


func _process(delta: float):
	if Engine.is_editor_hint():
		if entity_initializer is EntityInitializer:
			# TODO: Run as little as possible. Repeatedly constructs Rust structs in Rcs,
			# passes them around, then drops them.
			if EditorInterface.get_inspector().get_edited_object() == self:
				self.sync_with(entity_initializer.to_snapshot())

	if movement_tween is Tween:
		# We want to run these manually during the _process propagation.
		# Otherwise it would run in the idle time after that,
		# which ruins the current camera tracking.
		movement_tween.pause()
		movement_tween.custom_step(delta)


func is_animating() -> bool:
	if movement_tween is Tween and movement_tween.is_valid():
		return true
	return $AnimationPlayer.is_playing()


func is_important_animating() -> bool:
	if movement_tween is Tween and movement_tween.is_valid():
		return true
	return (
		$AnimationPlayer.is_playing()
		and (
			$AnimationPlayer.current_animation
			in [
				"Entity/Attack",
			]
		)
	)


func sync_with(snapshot: EntitySnapshot):
	if snapshot.get_passthrough() != "":
		# TODO: Figure out something smarter than string comparison
		if snapshot.get_passthrough() == "Axolotl":
			$AnimatedSprite3D.sprite_frames = preload(
				"res://Crawler/FloorView/Entity/axolotl/Axolotl.tres"
			)
		elif snapshot.get_passthrough().contains("Enemy2"):
			$AnimatedSprite3D.sprite_frames = preload(
				"res://Crawler/FloorView/Entity/tegu/TeguClone.tres"
			)
		elif snapshot.get_passthrough().contains("Enemy"):
			$AnimatedSprite3D.sprite_frames = preload(
				"res://Crawler/FloorView/Entity/gecko/Gecko.tres"
			)

	position = Vector3(snapshot.get_pos().x, 0, snapshot.get_pos().y)
	last_known_position = snapshot.get_pos()

	find_child("Debug").text = snapshot.get_passthrough()
	find_child("DebugHealth").text = str(snapshot.get_energy())

	$AnimationPlayer.play("RESET")
	$AnimationPlayer.seek(100, true)
	$AnimationPlayer.play(snapshot_to_idle_animation(snapshot))
	$AnimationPlayer.seek(100, true)
	$AnimationPlayer.stop()


func snapshot_to_idle_animation(snapshot):
	if snapshot.get_state_name() == "Knockdown":
		return "Entity/StateKnockdown"
	if snapshot.get_state_name() == "Hitstun":
		return "Entity/StateHitstun"
	if snapshot.get_state_name() == "RestrictedActions":
		return "Entity/StateCommitted"
	if snapshot.get_state_name() == "Committed":
		return "Entity/StateCommitted"
	if snapshot.get_state_name() == "Downed":
		return "Entity/StateDowned"
	if snapshot.get_state_name() == "Ok":
		# TODO: Indicate "moving next turn"
		return "Entity/StateOk"
	if snapshot.get_state_name() == "ConfirmCommand":
		return "Entity/StateOk"
	printerr("Unknown entity state ", snapshot.get_state_name())
	return "Entity/StateOk"
