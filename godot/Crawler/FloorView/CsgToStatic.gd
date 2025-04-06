@tool
extends Node3D

@export_tool_button("Bake All") var bake_button = bake_all
@export_tool_button("Manual Check") var check_button = manual_check
@export_tool_button("Clear All") var unspace_button = unbake_all


# assumes all csgs are children of meshes.
func bake_all():
	for i in range(5):
		var prism = $CSGPrisms.get_child(i)
		var mesh = $Meshes.get_child(i)

		prism.position = Vector3(i * 2, 0, 2)
		mesh.position = Vector3(-0.5, 0, -0.5)
		# Manual rotation by 90 degrees. Maybe better than an inaccurate PI?
		mesh.basis = Basis(Vector3.RIGHT, Vector3.UP / 2, Vector3.BACK)
		mesh.basis *= Basis(Vector3.RIGHT, Vector3.BACK, Vector3.DOWN)

		bake_csg_to_mesh(mesh, prism, i)


func bake_csg_to_mesh(mesh: MeshInstance3D, csg: CSGShape3D, hardcoded_case: int):
	var array_mesh: ArrayMesh = csg.bake_static_mesh().duplicate()

	var mdt = MeshDataTool.new()
	mdt.create_from_surface(array_mesh, 0)
	for vertex_i in range(mdt.get_vertex_count()):
		var vert = mdt.get_vertex(vertex_i)
		# don't touch the (to be) bottom face.
		if vert.z >= -0.9:
			continue
		# don't touch the corners.
		if vert.x in [0.0, 1.0] and vert.y in [0.0, 1.0]:
			continue

		# yeah whatever, don't need to be clever.
		if hardcoded_case == 0:
			# inner corner
			vert.x /= 2
			vert.y /= 2
		elif hardcoded_case == 1:
			# flat wall
			vert.y /= 2
		elif hardcoded_case == 2:
			# diagonal thin wall
			if vert.x + vert.y > 1:
				# Average towards the (1, 1) corner instead
				vert += Vector3(1, 1, 0)
			vert.x /= 2
			vert.y /= 2
		elif hardcoded_case == 3:
			# outer corner
			if vert.x > vert.y:
				vert += Vector3(1, 0, 0)
			else:
				vert += Vector3(0, 1, 0)
			vert.x /= 2
			vert.y /= 2
		elif hardcoded_case == 4:
			# full wall
			pass
		else:
			push_warning("unexpected case?")

		mdt.set_vertex(vertex_i, vert)

	# oh no! an extra allocation on this one time script! anyways.
	mesh.mesh = ArrayMesh.new()
	mdt.commit_to_surface(mesh.mesh)
	mesh.mesh.regen_normal_maps()


func unbake_all():
	for mesh in $Meshes.get_children():
		mesh.mesh = null


func manual_check():
	bake_all()
	for i in range(5):
		$Meshes.get_child(i).position += Vector3(2 * i, 0, 0)
