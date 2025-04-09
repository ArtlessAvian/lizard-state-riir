extends Node


func postprocess(hardcoded_case: int, surface: int, mdt: MeshDataTool):
	if surface != 0:
		return

	for vertex_i in range(mdt.get_vertex_count()):
		var vert = mdt.get_vertex(vertex_i)
		vert.x += sign(vert.x) * 0.01
		vert.z += sign(vert.z) * 0.01
		mdt.set_vertex(vertex_i, vert)
