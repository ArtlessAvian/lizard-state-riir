extends Camera3D

func _process(delta):
	keep_aspect = Camera3D.KEEP_WIDTH
	self.size = get_viewport().size.x / 24.0 / 1.0 # meters
	
