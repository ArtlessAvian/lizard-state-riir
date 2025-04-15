func postprocess(hardcoded_case: int, surface: int, mdt: MeshDataTool):
	if surface != 0:
		return

	for vertex_i in range(mdt.get_vertex_count()):
		var vert = mdt.get_vertex(vertex_i)
		var corner = get_connected_corner(vert, hardcoded_case)

		# bevel the top face only
		if vert.y >= 0.9:
			vert = vert.lerp(corner, 0.5)

		mdt.set_vertex(vertex_i, vert)

	for face_i in range(mdt.get_face_count()):
		if face_is_occluded(mdt, face_i, hardcoded_case):
			for tri_vert in range(3):
				var vertex_i = mdt.get_face_vertex(face_i, tri_vert)
				mdt.set_vertex(vertex_i, Vector3.DOWN * 100)


func get_connected_corner(vert: Vector3, hardcoded_case: int) -> Vector3:
	if hardcoded_case == 2:
		if vert.x + vert.z >= 0:
			return Vector3(0.5, vert.y, 0.5)

	if vert.x <= 0:
		if vert.z <= 0:
			return Vector3(-0.5, vert.y, -0.5)
		else:
			return Vector3(-0.5, vert.y, 0.5)
	else:
		if vert.z <= 0:
			return Vector3(0.5, vert.y, -0.5)
		else:
			return Vector3(0.5, vert.y, 0.5)


func face_is_occluded(mdt: MeshDataTool, face_i: int, hardcoded_case: int) -> bool:
	var a = mdt.get_vertex(mdt.get_face_vertex(face_i, 0))
	var b = mdt.get_vertex(mdt.get_face_vertex(face_i, 1))
	var c = mdt.get_vertex(mdt.get_face_vertex(face_i, 2))

	var normal = (a - c).cross(a - b)
	# intentional magic number
	var origin = Vector3(0, 0.5, 0)
	if hardcoded_case == 3:
		origin = Vector3(0.45, 0.5, 0.45)

	# normal points in the same direction as a - origin.
	return normal.dot(a - origin) > 0
