@tool
extends Node

@export_tool_button("Bake All") var bake_button = bake_all


func _ready() -> void:
	for i in range(5):
		var csg = $CSGs.get_child(i)
		csg.transform = Transform3D.IDENTITY
		csg.position = Vector3(i * 2, 0, 2)

	$GridMap.clear()
	for i in range(5):
		$GridMap.set_cell_item(Vector3i(2 * i, 0, 0), i)


func bake_all():
	var library: MeshLibrary
	if ResourceLoader.exists("res://Crawler/FloorView/MarchingSquaresMeshSymmetric.meshlib"):
		library = load("res://Crawler/FloorView/MarchingSquaresMeshSymmetric.meshlib")
	else:
		library = MeshLibrary.new()
		library.take_over_path("res://Crawler/FloorView/MarchingSquaresMeshSymmetric.meshlib")

	for i in range(5):
		var csg = $CSGs.get_child(i)
		library.create_item(i)
		library.set_item_mesh(i, bake_csg_to_mesh(csg, i))

	# Compressing since the output will be binary no matter what.
	print(
		error_string(
			ResourceSaver.save(library, library.resource_path, ResourceSaver.FLAG_COMPRESS)
		)
	)
	print(library.resource_path)


func bake_csg_to_mesh(csg: CSGShape3D, hardcoded_case: int) -> ArrayMesh:
	var array_mesh: ArrayMesh = csg.bake_static_mesh().duplicate()

	var mdt = MeshDataTool.new()
	mdt.create_from_surface(array_mesh, 0)
	for vertex_i in range(mdt.get_vertex_count()):
		var vert = mdt.get_vertex(vertex_i)
		var corner = get_connected_corner(vert, hardcoded_case)
		if corner != vert:
			var weighted = vert.lerp(corner, 0.5)
			mdt.set_vertex(vertex_i, weighted)
			prints("moving", vert, "to", corner)

	# oh no! an extra allocation on this one time script! anyways.
	var out = ArrayMesh.new()
	mdt.commit_to_surface(out)

	for surface in range(1, array_mesh.get_surface_count()):
		var noop = MeshDataTool.new()
		noop.create_from_surface(array_mesh, surface)
		noop.commit_to_surface(out)

	out.regen_normal_maps()
	return out


func get_connected_corner(vert: Vector3, hardcoded_case: int) -> Vector3:
	# only touch the top face
	if not (vert.y >= 0.9):
		return vert
	# don't touch the corners.
	if vert.x <= -0.5 or vert.x >= 0.5:
		if vert.z <= -0.5 or vert.z >= 0.5:
			return vert

	# yeah whatever, don't need to be clever.
	if hardcoded_case == 0:
		# inner corner
		return Vector3(-0.5, vert.y, -0.5)
	elif hardcoded_case == 1:
		# flat wall
		return Vector3(vert.x, vert.y, -0.5)
	elif hardcoded_case == 2:
		# diagonal thin wall
		if vert.x + vert.z > 0:
			return Vector3(0.5, vert.y, 0.5)
		return Vector3(-0.5, vert.y, -0.5)
	elif hardcoded_case == 3:
		# outer corner
		if vert.x < 0 and vert.z < 0:
			return Vector3(-0.5, vert.y, -0.5)
		elif vert.x > vert.z:
			return Vector3(0.5, vert.y, -0.5)
		else:
			return Vector3(-0.5, vert.y, 0.5)
	elif hardcoded_case == 4:
		# technically this is unreachable.
		# anyways, be the identity function.
		return vert
	else:
		push_warning("unexpected case?")
		# be the identity function.
		return vert
