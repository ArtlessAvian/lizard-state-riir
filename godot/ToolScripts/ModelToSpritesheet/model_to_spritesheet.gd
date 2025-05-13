@tool
extends Node

@export_tool_button("Write") var write_button = write


func write():
	var image = Image.create_empty(64 * 8, 64 * 8, false, Image.FORMAT_RGBA8)

	RenderingServer.force_draw()

	for facing in range(5):
		var viewport = $AngledCameras.get_child(facing) as Viewport
		image.blit_rect(
			viewport.get_texture().get_image(), Rect2i(0, 0, 64, 64), Vector2i(0, facing * 64)
		)

	image.save_png("res://ToolScripts/ModelToSpritesheet/temp.png")

	var preview = self.get_node_or_null("PreviewUV")
	if preview == null:
		preview = $PreviewBase.duplicate()
		preview.name = "PreviewUV"
		self.add_child(preview)
	preview.texture = ImageTexture.create_from_image(image)
