extends Node

var floor: Floor;

func _ready():
	floor = Floor.new();
	floor.add_entity();

func _process(delta):
	poll_input()
	
	var player = floor.get_player()
	$WorldRotate/InternalRotation/Entity.position = Vector3(player.get_pos().x, 0, player.get_pos().y)

func poll_input():
	if Input.is_action_just_pressed("move_left"):
		move_player(Vector2i.LEFT)
	if Input.is_action_just_pressed("move_up"):
		move_player(Vector2i.UP)
	if Input.is_action_just_pressed("move_down"):
		move_player(Vector2i.DOWN)
	if Input.is_action_just_pressed("move_right"):
		move_player(Vector2i.RIGHT)
	if Input.is_action_just_pressed("move_upleft"):
		move_player(Vector2i.UP + Vector2i.LEFT)
	if Input.is_action_just_pressed("move_upright"):
		move_player(Vector2i.UP + Vector2i.RIGHT)
	if Input.is_action_just_pressed("move_downleft"):
		move_player(Vector2i.DOWN + Vector2i.LEFT)
	if Input.is_action_just_pressed("move_downright"):
		move_player(Vector2i.DOWN + Vector2i.RIGHT)

func move_player(dir: Vector2i):
	var player = floor.get_player()
	var action : Action = floor.get_step_action(dir)
	var command = action.to_command(floor, player)
	floor = command.do_action(floor)
