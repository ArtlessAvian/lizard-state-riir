extends Camera3D

func _process(delta):
	keep_aspect = Camera3D.KEEP_WIDTH
	
	# pixels * (1 tile / 24 pixels) * (1 meter / tile)
	self.size = get_viewport().size.x / 24.0
	self.size /= 3.0 # zoom some extra
	
