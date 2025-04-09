@tool
extends GridMap

@export_tool_button("Reready") var reready_button = _ready
@export_tool_button("Bake Me") var bake_button = bake_me

@export var postprocessor: GDScript = null


func _ready() -> void:
	for i in range(get_child_count()):
		var csg = get_child(i)
		csg.transform = Transform3D.IDENTITY
		csg.position = Vector3(i * 2, 0, 0)

		self.set_cell_item(Vector3i(2 * i, 0, 2), i)


func bake_me():
	var library: MeshLibrary = self.mesh_library
	if library == null:
		library = MeshLibrary.new()

	var resource_path = "res://ToolScripts/CSGToMeshLibrary/Output/{0}.meshlib".format([self.name])
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

		mdt.commit_to_surface(out)

	out.regen_normal_maps()
	return out
