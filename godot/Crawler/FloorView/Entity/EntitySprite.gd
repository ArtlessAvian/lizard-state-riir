@tool
extends Sprite3D

@export var towards: Texture
@export var right: Texture
@export var away: Texture

# y is dropped for calculations.
@export var look_dir: Vector3 = Vector3.BACK
@export var look_dir_offset: float = 0  # full rotations


func _process(delta):
	var camera_direction
	var camera_up
	if get_viewport().get_camera_3d() is Camera3D and not Engine.is_editor_hint():
		camera_direction = -get_viewport().get_camera_3d().basis.z
		camera_up = get_viewport().get_camera_3d().basis.y
	else:
		camera_direction = Vector3.FORWARD
		camera_up = Vector3.UP

	self.transform.basis = Basis.looking_at(camera_direction, camera_up)
	spin_around(camera_direction)


func spin_around(camera_direction):
	if not self.get_parent() or not self.get_parent().get_parent():
		return  # we need to yoink their global transform since we threw ours out.

	var look_dir_local = self.get_parent().get_parent().global_transform.basis * look_dir
	look_dir_local = look_dir_local.rotated(Vector3.UP, look_dir_offset * 2 * PI)

	var cam_dir_drop_y = camera_direction * Vector3(1, 0, 1)
	var look_dir_drop_y = look_dir_local * Vector3(1, 0, 1)

	var angle = rad_to_deg(cam_dir_drop_y.angle_to(look_dir_drop_y))
	if angle < 30:
		self.flip_h = false
		self.texture = away
	elif angle > 150:
		self.flip_h = false
		self.texture = towards
	else:
		self.flip_h = cam_dir_drop_y.cross(look_dir_drop_y).y > 0
		self.texture = right
