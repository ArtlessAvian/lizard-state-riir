extends Node

var floor: Floor;

# Called when the node enters the scene tree for the first time.
func _ready():
	floor = Floor.new();
	floor.add_entity();

# Called every frame. 'delta' is the elapsed time since the previous frame.
var i = 0
func _process(delta):
	i += 1
	if i % 100 != 0:
		return
	
	var dir = Vector2i(randi_range(-1, 1), randi_range(-1, 1))
	var action = floor.get_step_action(dir)
	var player = floor.get_player()
	print(player.get_pos())
	$WorldRotate/InternalRotation/Entity.position = Vector3(player.get_pos().x, 0, player.get_pos().y)
	
	var command = action.to_command(floor, player)
	floor = command.do_action(floor)
