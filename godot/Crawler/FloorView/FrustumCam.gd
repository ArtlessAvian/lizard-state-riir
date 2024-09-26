@tool
extends Camera3D

const TILE_APPARENT_WIDTH: float = 24
const LARGE_DISTANCE: float = 1000

@export_range(0, 5) var screen_pixels_per_pixel: int = 3
@export_range(0, 24) var tile_apparent_height: float = 16


func _process(_delta):
	if not Engine.is_editor_hint():
		var viewport = (
			get_viewport()
			#if not Engine.is_editor_hint()
			#else EditorInterface.get_editor_viewport_3d().get_viewport()
		)
			
		self.size = viewport.size.x / TILE_APPARENT_WIDTH
		self.size /= screen_pixels_per_pixel  # zoom some extra

	self.position = Vector3(
		0, LARGE_DISTANCE * tile_apparent_height, LARGE_DISTANCE * TILE_APPARENT_WIDTH
	)

	self.frustum_offset = Vector2.DOWN * -self.position.y
	self.near = self.position.z
	self.far = self.position.z + 100

	self.position += self.position * 48 / self.position.z
