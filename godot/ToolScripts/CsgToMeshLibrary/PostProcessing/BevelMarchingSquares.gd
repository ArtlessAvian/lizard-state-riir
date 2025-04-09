func postprocess(hardcoded_case: int, surface: int, mdt: MeshDataTool):
	if surface != 0:
		return

	for vertex_i in range(mdt.get_vertex_count()):
		var vert = mdt.get_vertex(vertex_i)
		var corner = get_connected_corner(vert, hardcoded_case)
		if corner != vert:
			var weighted = vert.lerp(corner, 0.5)
			mdt.set_vertex(vertex_i, weighted)


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
