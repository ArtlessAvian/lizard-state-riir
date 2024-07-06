extends Node3D


func sync_with(entity: Entity):
	position = Vector3(entity.get_pos().x, 0, entity.get_pos().y)
	find_child("Debug").text = entity.get_debug()
	find_child("DebugHealth").text = str(entity.get_health())
