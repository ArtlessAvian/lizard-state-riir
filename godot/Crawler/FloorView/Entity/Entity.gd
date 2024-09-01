extends Node3D

var last_known_position: Vector2i


func sync_with(snapshot: EntitySnapshot):
	position = Vector3(snapshot.get_pos().x, 0, snapshot.get_pos().y)
	last_known_position = snapshot.get_pos()

	find_child("Debug").text = snapshot.get_debug()
	find_child("DebugHealth").text = str(snapshot.get_energy())
