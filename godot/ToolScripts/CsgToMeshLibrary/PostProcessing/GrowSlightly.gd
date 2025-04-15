extends Node


func postprocess(hardcoded_case: int, surface: int, mdt: MeshDataTool):
	if surface != 0:
		return

	for vertex_i in range(mdt.get_vertex_count()):
		var vert = mdt.get_vertex(vertex_i)
		vert.x += sign(vert.x) * 0.001
		vert.z += sign(vert.z) * 0.001
		mdt.set_vertex(vertex_i, vert)

	for face_i in range(mdt.get_face_count()):
		if mdt.get_face_normal(face_i) == Vector3.UP:
			# Do not discard this face.
			continue

		# Discard this face.
		for tri_vert in range(3):
			var vert_i = mdt.get_face_edge(face_i, tri_vert)
			# arbitrary vertex, as long as its consistent.
			mdt.set_vertex(vert_i, Vector3.DOWN)
