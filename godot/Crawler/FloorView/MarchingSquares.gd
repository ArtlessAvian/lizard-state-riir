@tool
extends GridMap

@export var source: GridMap
@export var tile: int
@export_tool_button("Reready") var reready = _ready

const MESHES = [-1, 0, 0, 1, 0, 1, 2, 3, 0, 2, 1, 3, 1, 3, 3, 4]
const Y_ROT = [0, 0, 3, 0, 2, 2, 2, 0, 1, 0, 3, 3, 1, 2, 1, 0]
const Y_ROT_TO_ORIENTATION = [0, 10, 16, 22]

var dirty = Array()


func _ready() -> void:
	self.clear()

	for cell in source.get_used_cells_by_item(tile):
		dirty.append(cell)


func _process(_delta: float) -> void:
	var my_dirty: Dictionary = Dictionary()
	for parent_cell in dirty:
		my_dirty[parent_cell] = null
		my_dirty[parent_cell - Vector3i(1, 0, 0)] = null
		my_dirty[parent_cell - Vector3i(0, 0, 1)] = null
		my_dirty[parent_cell - Vector3i(1, 0, 1)] = null
	dirty.clear()

	for cell in my_dirty.keys():
		reset_my_cell(cell)


func mark_dirty(parent_cell: Vector3i):
	dirty.append(parent_cell)


func reset_my_cell(cell: Vector3i):
	var pattern = 0
	for bit in range(4):
		var dx = bit % 2
		var dz = bit / 2  # Expected Integer Division
		var offset = cell + Vector3i(dx, 0, dz)
		if source.get_cell_item(offset) == tile:
			pattern |= 1 << bit
	if pattern != 0:
		self.set_cell_item(cell, MESHES[pattern], Y_ROT_TO_ORIENTATION[Y_ROT[pattern]])
