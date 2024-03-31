@tool
extends Sprite3D

@export var towards: Texture
@export var right: Texture
@export var away: Texture

# y is dropped for calculations.
@export var look_at: Vector3 = Vector3.BACK

# 5 pixels clip below the floor. The floor has slope sqrt(5)/2.
# TODO: Maybe calculate based on need rather than constant?
# But then there's ordering issues if not constant.
# Also seems expensive to calculate if sprites aren't packed well.
const SCOOT_AMOUNT: float = 5.0 / 24.0 * sqrt(5) / 2.0

func _process(delta):
	if get_viewport().get_camera_3d() is Camera3D:
		var camera_direction = get_viewport().get_camera_3d().basis.z
		spin_around(camera_direction)
		
		# Secretly scoot a little towards the camera to unclip some pixels.
		# Not visible orthographically / doesn't affect apparent size.
		# self.transform.origin = camera_direction.normalized() * SCOOT_AMOUNT

func spin_around(camera_direction):
	if not self.get_parent() or not self.get_parent().get_parent():
		return # we need to yoink their global transform since we threw ours out.

	var look_at_local = self.get_parent().get_parent().global_transform.basis * look_at
	
	var cam_dir_drop_y = camera_direction * Vector3(1, 0, 1)
	var look_at_drop_y = look_at_local * Vector3(1, 0, 1)
	
	var angle = cam_dir_drop_y.angle_to(look_at_drop_y)
	if angle < 45 * PI / 180.0:
		self.flip_h = false
		self.texture = towards
	elif angle < 135 * PI / 180.0:
		self.flip_h = cam_dir_drop_y.cross(look_at_drop_y).y < 0
		self.texture = right
	else:
		self.flip_h = false
		self.texture = away
