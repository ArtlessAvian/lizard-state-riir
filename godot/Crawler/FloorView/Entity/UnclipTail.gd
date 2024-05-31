@tool
extends Sprite3D


func _process(_delta: float) -> void:
	self.texture = get_parent().texture
	self.flip_h = get_parent().flip_h
