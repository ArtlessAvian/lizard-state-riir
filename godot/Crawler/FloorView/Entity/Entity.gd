@tool
extends Node3D

# Read-only for tool/editor stuff.
@export var entity_initializer: EntityInitializer

# The actual logic and stuff.
var last_known_position: Vector2i


func _ready():
	if Engine.is_editor_hint():
		if entity_initializer is EntityInitializer:
			self.sync_with(entity_initializer.to_snapshot())


func _process(_delta: float):
	if Engine.is_editor_hint():
		if entity_initializer is EntityInitializer:
			# TODO: Run as little as possible. Repeatedly constructs Rust structs in Rcs,
			# passes them around, then drops them.
			if EditorInterface.get_inspector().get_edited_object() == self:
				self.sync_with(entity_initializer.to_snapshot())


func sync_with(snapshot: EntitySnapshot):
	position = Vector3(snapshot.get_pos().x, 0, snapshot.get_pos().y)
	last_known_position = snapshot.get_pos()

	find_child("Debug").text = snapshot.get_debug()
	find_child("DebugHealth").text = str(snapshot.get_energy())

	$AnimationPlayer.play("RESET")
	$AnimationPlayer.seek(100, true)
	$AnimationPlayer.play(snapshot_to_idle_animation(snapshot))


func snapshot_to_idle_animation(snapshot):
	if snapshot.get_state_name() == "Knockdown":
		return "Entity/StateKnockdown"
	if snapshot.get_state_name() == "Hitstun":
		return "Entity/StateHitstun"
	if snapshot.get_state_name() == "Committed":
		return "Entity/StateCommitted"
	return "Entity/StateOk"
