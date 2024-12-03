@tool
extends AnimatedSprite3D

# y is dropped for calculations.
@export var look_dir_offset: float = 0  # full rotations

# This could run all the time, but it would save rotation and cause
# version control noise.
@export var rotate_in_editor: bool = false:
	set(value):
		rotate_in_editor = value
		update_configuration_warnings()


func _get_configuration_warnings() -> PackedStringArray:
	if rotate_in_editor:
		return ["Uncheck rotate_in_editor before committing!"]
	return []


func _process(delta):
	var camera_direction: Vector3
	var camera_up: Vector3
	if not Engine.is_editor_hint() and get_viewport().get_camera_3d() is Camera3D:
		var camera = get_viewport().get_camera_3d()
		camera_direction = -camera.global_basis.z
		camera_up = camera.global_basis.y
	elif Engine.is_editor_hint() and rotate_in_editor:
		var camera = EditorInterface.get_editor_viewport_3d().get_camera_3d()
		camera_direction = -camera.global_basis.z
		camera_up = camera.global_basis.y
	else:
		camera_direction = Vector3.FORWARD
		camera_up = Vector3.UP

	self.global_basis = Basis.looking_at(camera_direction, camera_up)
	# HACK: Alternatively, grow taller to maintain visual height, without losing coordinate system
	#var cam_dir_drop_y = camera_direction * Vector3(1, 0, 1)
	#self.global_basis = Basis.looking_at(cam_dir_drop_y, camera_up)
	#self.global_basis *= Basis.from_scale(Vector3(1, 1 / sqrt(1 - camera_direction.y * camera_direction.y), 1))

	spin_around(camera_direction)

	#self.transparency = clamp((self.position.y - 4)/2.0, 0, 1)


func spin_around(camera_direction):
	if not self.get_parent():
		return  # we need to yoink their global transform since we threw ours out.

	var held_frame = self.frame

	var local_forward = self.get_parent().global_basis * Vector3.MODEL_FRONT
	local_forward = local_forward.rotated(Vector3.UP, look_dir_offset * 2 * PI)

	var cam_dir_drop_y = camera_direction * Vector3(1, 0, 1)
	var look_dir_drop_y = local_forward * Vector3(1, 0, 1)

	var angle = rad_to_deg(cam_dir_drop_y.angle_to(look_dir_drop_y))
	if angle < 30:
		self.flip_h = false
		self.animation = "Away"
	elif angle > 150:
		self.flip_h = false
		self.animation = "Towards"
	else:
		self.flip_h = cam_dir_drop_y.cross(look_dir_drop_y).y > 0
		self.animation = "Right"

	self.frame = held_frame
