@tool
extends Node3D

@export_tool_button("Bake All") var bake_button = bake_all
@export_tool_button("Unbake All") var unspace_button = unbake_all


func find_mesh_with_child_csg(maybe_mesh: Node, callback: Callable):
	if maybe_mesh is MeshInstance3D:
		find_child_csg(maybe_mesh, callback)
	else:
		for child in maybe_mesh.get_children():
			find_mesh_with_child_csg(child, callback)


func find_child_csg(mesh: MeshInstance3D, callback: Callable):
	for child in mesh.get_children():
		if child is CSGShape3D:
			callback.call(mesh, child)


# assumes all csgs are children of meshes.
func bake_all():
	unspace_all()
	find_mesh_with_child_csg(self, bake_csg_to_mesh)


func unspace_all():
	for child in self.get_children():
		child.position = Vector3(-0.5, 0, -0.5)


func bake_csg_to_mesh(mesh: MeshInstance3D, csg: CSGShape3D):
	csg.basis = Quaternion.IDENTITY
	mesh.mesh = csg.bake_static_mesh()
	mesh.basis = Basis(Vector3.RIGHT, Vector3.UP / 2, Vector3.BACK)
	mesh.basis *= Basis(Vector3.RIGHT, Vector3.BACK, Vector3.DOWN)


func unbake_all():
	space_all()
	find_mesh_with_child_csg(self, clear_mesh)


func space_all():
	for child_i in self.get_child_count():
		var child = self.get_child(child_i)
		child.position.x = child_i


func clear_mesh(mesh: MeshInstance3D, csg: CSGShape3D):
	mesh.mesh = null
