@tool
extends Node3D
class_name Map

@export var is_history = false
@export var history_of: Map

const DITHER_SHADER = preload("res://Crawler/FloorView/dither.gdshader")


func _ready() -> void:
	if is_history and not Engine.is_editor_hint():
		var yeah = ShaderMaterial.new()
		yeah.shader = DITHER_SHADER

		var mesh_library: MeshLibrary = history_of.get_node("Floors").mesh_library.duplicate()
		apply_shader(mesh_library, yeah)
		$Floors.mesh_library = mesh_library

		mesh_library = history_of.get_node("Walls").mesh_library.duplicate(false)
		apply_shader(mesh_library, yeah)
		$Walls.mesh_library = mesh_library

		mesh_library = history_of.get_node("WallsMarching").mesh_library.duplicate(false)
		apply_shader(mesh_library, yeah)
		$WallsMarching.mesh_library = mesh_library

		mesh_library = history_of.get_node("FloorsMarching").mesh_library.duplicate(false)
		apply_shader(mesh_library, yeah)
		$FloorsMarching.mesh_library = mesh_library


func apply_shader(mesh_library: MeshLibrary, yeah: ShaderMaterial):
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

			mesh.surface_set_material(0, material)
			material.render_priority += -1
			while material.next_pass != null:
				var next = material.next_pass.duplicate()
				material.next_pass = next
				material = next
				material.render_priority += -1

			material.next_pass = yeah


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
