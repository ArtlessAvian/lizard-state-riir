@tool
extends Camera3D

@export var screen_pixels_per_pixel: int = 3

enum CameraSkew { PRESERVE_ALL, PRESERVE_Y, PRESERVE_XZ }
@export var do_skew_world: CameraSkew = CameraSkew.PRESERVE_ALL

# apparent/screenspace height vs width.
@export_range(0, 24) var tile_apparent_height: float = 16
const tile_apparent_width: float = 24

# the *cameras* rotation. if tracking an object, it should appear to move clockwise.
@export_range(-360, 360) var y_rotation_degrees: float = 0

# TODO: Decide if this is a good idea:
# look_at_target can be replaced with a RemoteTransform3D pushing a global position to this camera.
# The RemoteTransform can be moved in local coordinates. This moves the responsibility to the
# RemoteTransform to move the camera, and multiple may *try* to assert their position which
# won't work.

# Target in WorldSkew's local coordinates.
@export var look_at_target: Vector3 = Vector3.ZERO

@export var demo_mode = false


func _process(delta):
	# TODO: Temporary.
	look_at_target = %Entity.global_position.lerp(look_at_target, exp(-delta * 5))

	if not Engine.is_editor_hint():
		adjust_size()

		if demo_mode:
			self.y_rotation_degrees += delta * 360.0 / 10.0
			if int(self.y_rotation_degrees) % (360 * 3) < 360:
				self.do_skew_world = CameraSkew.PRESERVE_ALL
			elif int(self.y_rotation_degrees) % (360 * 3) < 360 * 2:
				self.do_skew_world = CameraSkew.PRESERVE_Y
			else:
				self.do_skew_world = CameraSkew.PRESERVE_XZ

	if $"../WorldSkew" is Node3D:
		var skew_node: Node3D = $"../WorldSkew"

		if !self.current or do_skew_world == CameraSkew.PRESERVE_ALL:
			skew_node.transform = Transform3D.IDENTITY
			self.transform.basis = Basis(Vector3.UP, y_rotation_degrees * PI / 180.0)
			self.transform.basis *= Basis(
				Vector3.RIGHT, -asin(tile_apparent_height / tile_apparent_width)
			)
			self.transform.origin = look_at_target + (self.transform.basis * (Vector3.BACK * 20))
		elif do_skew_world == CameraSkew.PRESERVE_Y:
			skew_node.transform = get_skew()
			self.transform = Transform3D(Basis.IDENTITY, Vector3.BACK * 20)
		elif do_skew_world == CameraSkew.PRESERVE_XZ:
			var ratio = min(tile_apparent_height / tile_apparent_width, 1 - 1e-15)

			self.transform.basis = Basis(Vector3.UP, y_rotation_degrees * PI / 180.0)
			self.transform.basis *= Basis(Vector3.RIGHT, -asin(ratio))
			self.transform.origin = look_at_target + (self.transform.basis * (Vector3.BACK * 20))

			# alternatively, cos(asin(ratio))
			var projection_scaling = max(1e-9, sqrt(1 - ratio * ratio))
			var y_axis = self.transform.basis.y * projection_scaling
			skew_node.transform.basis = Basis(Vector3.RIGHT, y_axis, Vector3.BACK)
			skew_node.transform.origin = Vector3.ZERO
		else:
			print("fallthrough")


func adjust_size():
	keep_aspect = Camera3D.KEEP_WIDTH
	# pixels * (1 tile / 24 pixels) * (1 meter / tile)
	self.size = get_viewport().size.x / tile_apparent_width
	self.size /= screen_pixels_per_pixel  # zoom some extra


func get_skew() -> Transform3D:
	var ratio = tile_apparent_height / tile_apparent_width

	var translate_xz = Transform3D(Basis.IDENTITY, -look_at_target)
	var rotate_around_y = Basis(Vector3.UP, -y_rotation_degrees * PI / 180.0)
	var rotate_yz_but_maintain_up = Basis(
		Vector3.RIGHT,
		# Points (almost) up.
		# Small z component to suggest drawing order. Increase if you see z-fighting.
		Vector3(0, max(sqrt(1 - ratio * ratio), 1e-9), 1e-5),
		Vector3(0, -ratio, max(sqrt(1 - ratio * ratio), 1e-9)),
	)

	var out = (
		Transform3D(rotate_yz_but_maintain_up, Vector3.ZERO)
		* Transform3D(rotate_around_y, Vector3.ZERO)
		* translate_xz
	)

	#assert(out.basis.y.x == 0 and out.basis.y.z == 0)
	return out
