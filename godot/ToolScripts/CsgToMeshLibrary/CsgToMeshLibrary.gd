@tool
extends GridMap

@export_tool_button("Reready") var reready_button = _ready
@export_tool_button("Bake Me") var bake_button = bake_me

@export var postprocessor: GDScript = null


func _ready() -> void:
	self.clear()
	for i in range(get_child_count()):
		var csg = get_child(i)
		csg.transform = Transform3D.IDENTITY
		csg.position = Vector3(i * 2, 0, 0)

		self.set_cell_item(Vector3i(2 * i, 0, 2), i)


func bake_me():
	var library: MeshLibrary = self.mesh_library
	if library == null:
		library = MeshLibrary.new()

	var resource_path = "res://ToolScripts/CsgToMeshLibrary/Output/{0}.meshlib".format([self.name])
	library.take_over_path(resource_path)

	for i in range(get_child_count()):
		var csg: CSGShape3D = get_child(i)
		library.create_item(i)
		library.set_item_mesh(i, bake_csg_to_mesh(csg, i))

	# Compressing since the output will be binary no matter what.
	var status = ResourceSaver.save(library, library.resource_path, ResourceSaver.FLAG_COMPRESS)
	print(error_string(status))
	print(library.resource_path)


func bake_csg_to_mesh(csg: CSGShape3D, hardcoded_case: int) -> ArrayMesh:
	var out = ArrayMesh.new()

	var array_mesh: ArrayMesh = csg.bake_static_mesh().duplicate()
	for surface_i in range(array_mesh.get_surface_count()):
		var mdt = MeshDataTool.new()
		mdt.create_from_surface(array_mesh, surface_i)

		if postprocessor != null:
			var instance = postprocessor.new()
			instance.postprocess(hardcoded_case, surface_i, mdt)
		fix_normals(mdt)
		mdt.commit_to_surface(out)

	out.regen_normal_maps()
	return out


func fix_normals(mdt: MeshDataTool):
	# Zero out stuff before summing.
	for i in range(mdt.get_vertex_count()):
		mdt.set_vertex_normal(i, Vector3.ZERO)

	# Source: Godot official docs
	# https://docs.godotengine.org/en/stable/tutorials/3d/procedural_geometry/meshdatatool.html

	# Calculate vertex normals, face-by-face.
	for i in range(mdt.get_face_count()):
		# Get the index in the vertex array.
		var a = mdt.get_face_vertex(i, 0)
		var b = mdt.get_face_vertex(i, 1)
		var c = mdt.get_face_vertex(i, 2)
		# Get vertex position using vertex index.
		var ap = mdt.get_vertex(a)
		var bp = mdt.get_vertex(b)
		var cp = mdt.get_vertex(c)
		# Calculate face normal.
		var n = (bp - cp).cross(ap - bp).normalized()
		# Add face normal to current vertex normal.
		# This will not result in perfect normals, but it will be close.
		mdt.set_vertex_normal(a, n + mdt.get_vertex_normal(a))
		mdt.set_vertex_normal(b, n + mdt.get_vertex_normal(b))
		mdt.set_vertex_normal(c, n + mdt.get_vertex_normal(c))

	# Run through vertices one last time to normalize normals
	for i in range(mdt.get_vertex_count()):
		var v = mdt.get_vertex_normal(i).normalized()
		mdt.set_vertex_normal(i, v)
