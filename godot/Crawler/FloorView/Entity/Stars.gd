@tool
extends Node3D

# IM A MEMBER OF STARS

@export var pixel_snap_in_editor: bool = false:
	set(value):
		pixel_snap_in_editor = value
		update_configuration_warnings()
@export var animate_in_editor: bool = false:
	set(value):
		animate_in_editor = value
		update_configuration_warnings()

@export var apparent_fps: int = 4
@export var distinct_frames: int = 3
@export var radius: float = 0.4


func _get_configuration_warnings() -> PackedStringArray:
	if pixel_snap_in_editor:
		return ["Uncheck pixel_snap_in_editor before committing!"]
	if animate_in_editor:
		return ["Uncheck animate_in_editor before committing!"]
	return []


func _process(delta: float) -> void:
	var camera_basis: Basis
	if not Engine.is_editor_hint() and get_viewport().get_camera_3d() is Camera3D:
		var camera = get_viewport().get_camera_3d()
		camera_basis = camera.global_basis
	elif Engine.is_editor_hint() and pixel_snap_in_editor:
		var camera = EditorInterface.get_editor_viewport_3d().get_camera_3d()
		camera_basis = camera.global_basis
	else:
		camera_basis = Basis.IDENTITY

	# intentional snapping to 4 fps
	var frame = Time.get_ticks_msec() * apparent_fps / 1000
	if not animate_in_editor and Engine.is_editor_hint():
		frame = 0

	for child in self.get_children():
		var phase_from_index = (2 * PI / self.get_child_count()) * child.get_index()

		var phase_from_time = float(frame) / distinct_frames * 2 * PI / self.get_child_count()

		var real_position = (
			radius * Vector3.RIGHT.rotated(Vector3.UP, phase_from_time + phase_from_index)
		)
		var camera_snapped = (
			camera_basis * (((camera_basis.inverse() * real_position) * 24).round() / 24)
		)
		child.position = camera_snapped
