extends Node3D

var last_known_position: Vector2i


func sync_with(entity: Entity):
	position = Vector3(entity.get_pos().x, 0, entity.get_pos().y)
	last_known_position = entity.get_pos()

	find_child("Debug").text = entity.get_debug()
	find_child("DebugHealth").text = str(entity.get_energy())
