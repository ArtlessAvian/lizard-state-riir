@tool
extends Node3D

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	if get_parent():
		self.transform.basis = Basis.IDENTITY
		self.position = get_parent().global_position
