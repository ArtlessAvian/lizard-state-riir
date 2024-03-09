extends Node2D


# Called when the node enters the scene tree for the first time.
func _ready():
	var yay = Floor.new();
	yay.add_entity();
	yay.go_right();
	yay.go_right();
	yay.go_right();
	yay.go_right();
	yay.go_right();


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	pass
