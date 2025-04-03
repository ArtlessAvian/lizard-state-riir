@tool
extends GridMap

@export var walls: GridMap
@export_tool_button("Reready") var reready = _ready

const MESHES = [-1, 0, 0, 1, 0, 1, 2, 3, 0, 2, 1, 3, 1, 3, 3, 4]
const Y_ROT = [0, 0, 3, 0, 2, 2, 2, 0, 1, 0, 3, 3, 1, 2, 1, 0]
const Y_ROT_TO_ORIENTATION = [0, 10, 16, 22]


func _ready() -> void:
	self.clear()

	var check_me: Dictionary = Dictionary()
	for cell in walls.get_used_cells_by_item(1):
		check_me[cell] = null
		check_me[cell - Vector3i(1, 0, 0)] = null
		check_me[cell - Vector3i(0, 0, 1)] = null
		check_me[cell - Vector3i(1, 0, 1)] = null

	for cell in check_me.keys():
		var pattern = 0
		for bit in range(4):
			var dx = bit % 2
			var dz = bit / 2  # Expected Integer Division
			var offset = cell + Vector3i(dx, 0, dz)
			if walls.get_cell_item(offset) == 1:
				pattern |= 1 << bit
		if pattern != 0:
			self.set_cell_item(cell, MESHES[pattern], Y_ROT_TO_ORIENTATION[Y_ROT[pattern]])
		#self.set_cell_item(cell, 5)

	#for i in range(16):
	#self.set_cell_item(Vector3(2 * i, 0, 30), MESHES[i], Y_ROT_TO_ORIENTATION[Y_ROT[i]])
