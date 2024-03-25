@tool
extends Camera3D

# if false, moves the camera. if true, moves the world s.t. up is Y+
@export var do_skew_world = false

# apparent/screenspace height vs width.
@export_range(0, 24)
var tile_apparent_height: float = 16
const tile_apparent_width: float = 24

# the *cameras* rotation. if tracking an object, it should appear to move clockwise.
@export_range(-360, 360)
var y_rotation_degrees: float = 0

@export var look_at_target: Vector3 = Vector3.ZERO
@export var camera_distance = 20

func _process(delta):
	if not Engine.is_editor_hint():
		adjust_size()
	
	if $"../WorldSkew" is Node3D:
		var skew_node: Node3D = $"../WorldSkew";

		if do_skew_world:
			skew_node.transform = get_skew()
			self.transform = Transform3D(Basis.IDENTITY, Vector3.BACK * 20)
		else:
			skew_node.transform = Transform3D.IDENTITY
			self.transform.basis = Basis(Vector3.UP, y_rotation_degrees * PI / 180.0)
			self.transform.basis *= Basis(Vector3.RIGHT, -asin(tile_apparent_height/tile_apparent_width))
			self.transform.origin = look_at_target + (self.transform.basis * (Vector3.BACK * 20))

func adjust_size():
	keep_aspect = Camera3D.KEEP_WIDTH
	# pixels * (1 tile / 24 pixels) * (1 meter / tile)
	self.size = get_viewport().size.x / tile_apparent_width
	self.size /= 3.0 # zoom some extra

func get_skew() -> Transform3D:
	var ratio = tile_apparent_height/tile_apparent_width
		
	var translate_xz = Transform3D(Basis.IDENTITY, -look_at_target)
	var rotate_around_y = Basis(Vector3.UP, -y_rotation_degrees * PI/180.0)
	var rotate_yz_but_maintain_up = Basis(
			Vector3.RIGHT,
			# Points (almost) up. Small z component to suggest drawing order.
			Vector3(0, max(sqrt(1 - ratio * ratio), 1e-9), 1e-6),
			Vector3(0, -ratio, max(sqrt(1 - ratio * ratio), 1e-9)),
		)
	
	var out = Transform3D(rotate_yz_but_maintain_up, Vector3.ZERO) * \
			Transform3D(rotate_around_y, Vector3.ZERO) * \
			translate_xz

	#assert(out.basis.y.x == 0 and out.basis.y.z == 0)
	return out
