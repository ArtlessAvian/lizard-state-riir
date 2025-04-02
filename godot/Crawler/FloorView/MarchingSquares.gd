@tool
extends GridMap

@export var sibling: GridMap
@export_tool_button("Reready") var reready = _ready


func _ready() -> void:
	self.clear()

	var check_me: Dictionary = Dictionary()
	for cell in sibling.get_used_cells_by_item(1):
		check_me[cell] = null
		check_me[cell - Vector3i(1, 0, 0)] = null
		check_me[cell - Vector3i(0, 0, 1)] = null
		check_me[cell - Vector3i(1, 0, 1)] = null

	for cell in check_me.keys():
		var which = 0
		for bit in range(4):
			var dx = bit % 2
			var dz = bit / 2  # Expected Integer Division
			var offset = cell + Vector3i(dx, 0, dz)
			if sibling.get_cell_item(offset) == 1:
				which |= 1 << bit
		self.set_cell_item(cell, which)
		#self.set_cell_item(cell, 5)


# TODO: Only update when dirty!!!
func _process(delta: float) -> void:
	_ready()
