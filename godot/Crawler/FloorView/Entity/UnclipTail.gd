@tool
extends Sprite3D


func _process(_delta: float) -> void:
	self.texture = get_parent().sprite_frames.get_frame_texture(
		get_parent().animation, get_parent().frame
	)
	self.flip_h = get_parent().flip_h
