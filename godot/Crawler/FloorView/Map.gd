@tool
extends Node3D
class_name Map

@export var is_history = false
@export var history_of: Map

const DITHER_SHADER = preload("res://Crawler/FloorView/dither.gdshader")


func _ready() -> void:
	if is_history and not Engine.is_editor_hint():
		var mesh_library: MeshLibrary = history_of.get_node("Floors").mesh_library.duplicate()
		apply_shader(mesh_library)
		$Floors.mesh_library = mesh_library

		mesh_library = history_of.get_node("Walls").mesh_library.duplicate(false)
		apply_shader(mesh_library)
		$Walls.mesh_library = mesh_library

		mesh_library = history_of.get_node("WallsMarching").mesh_library.duplicate(false)
		apply_shader(mesh_library)
		$WallsMarching.mesh_library = mesh_library

		mesh_library = history_of.get_node("FloorsMarching").mesh_library.duplicate(false)
		apply_shader(mesh_library)
		$FloorsMarching.mesh_library = mesh_library


func apply_shader(mesh_library: MeshLibrary):
	for id in mesh_library.get_item_list():
		var mesh: Mesh = mesh_library.get_item_mesh(id).duplicate()
		mesh_library.set_item_mesh(id, mesh)

		for surface in range(mesh.get_surface_count()):
			var material: Material = mesh.surface_get_material(surface)
			if material == null:
				material = StandardMaterial3D.new()
				#material.albedo_color = Color.SADDLE_BROWN
			else:
				material = material.duplicate()

			var prepend_shader: ShaderMaterial = ShaderMaterial.new()
			prepend_shader.shader = DITHER_SHADER
			prepend_shader.render_priority = prepend_shader.RENDER_PRIORITY_MAX
			prepend_shader.set_shader_parameter("scoot_direction_worldspace", -self.position * 0.5)
			prepend_shader.next_pass = material

			mesh.surface_set_material(surface, prepend_shader)


func clear():
	$Floors.clear()
	$Walls.clear()
	$WallsMarching.clear()
	$FloorsMarching.clear()


func add_vision(vision: Dictionary):
	for pos in vision:
		if vision[pos]:
			$Floors.set_cell_item(Vector3i(pos.x, 0, pos.y), 0)
			$FloorsMarching.mark_dirty(Vector3i(pos.x, 0, pos.y))
		else:
			$Walls.set_cell_item(Vector3i(pos.x, 0, pos.y), 1)
			$WallsMarching.mark_dirty(Vector3i(pos.x, 0, pos.y))

	#$WallsMarching._ready()
