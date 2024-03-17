extends Node2D

var floor: Floor;

# Called when the node enters the scene tree for the first time.
func _ready():
	floor = Floor.new();
	floor.add_entity();

# Called every frame. 'delta' is the elapsed time since the previous frame.
var i = 0
func _process(delta):
	print(i)
	i += 1
	
	var player = floor.get_player()
	var action = floor.get_action();
	
	var command = action.to_command(floor, player)
	floor = command.do_action(floor)
