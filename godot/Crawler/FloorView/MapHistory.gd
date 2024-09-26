@tool
extends GridMap

# TODO: Check for VisualInstance3D::sorting_offset equivalent for GridMaps.
# This is fine in the meantime.


func _process(delta: float) -> void:
	if get_viewport().get_camera_3d() is Camera3D:
		self.position = -get_viewport().get_camera_3d().global_basis.z * 2000
