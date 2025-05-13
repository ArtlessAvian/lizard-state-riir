@tool
extends Camera3D


func _process(delta: float) -> void:
	self.global_transform = Transform3D.IDENTITY

	# first rotate
	self.global_transform *= Transform3D(
		Basis(Vector3.UP, PI / 4 * self.get_parent().get_index()), Vector3.ZERO
	)
	# then tilt up (the camera tilts down since it points backwards)
	self.global_transform *= Transform3D(Basis(Vector3.RIGHT, asin(-2.0 / 3)), Vector3.ZERO)
	# then move "forward" (the camera moves backwards since it points backwards)
	self.global_transform *= Transform3D(Basis.IDENTITY, Vector3(0, 0, 10))
