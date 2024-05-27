extends Node3D

var floor: Floor

var player_id: EntityId


# Called when the node enters the scene tree for the first time.
func _ready():
	floor = Floor.new()
	# HACK: Temporary.
	floor.set_map_2d($Map)

	player_id = floor.add_entity_at(Vector2i.ZERO, true)
	$Floor.id_to_node[player_id] = find_child("Entity")

	#floor.add_entity_at(Vector2i(-3, 0), false)
	#floor.add_entity_at(Vector2i(-3, -1), false)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	poll_input(delta)
	floor.take_npc_turn()

	$Floor._process_floor(delta, floor)


func poll_input(delta):
	if $Floor.desynced_from_floor:
		return

	if Input.is_action_pressed("move_left"):
		move_player(Vector2i.LEFT)
	if Input.is_action_pressed("move_up"):
		move_player(Vector2i.UP)
	if Input.is_action_pressed("move_down"):
		move_player(Vector2i.DOWN)
	if Input.is_action_pressed("move_right"):
		move_player(Vector2i.RIGHT)
	if Input.is_action_pressed("move_upleft"):
		move_player(Vector2i.UP + Vector2i.LEFT)
	if Input.is_action_pressed("move_upright"):
		move_player(Vector2i.UP + Vector2i.RIGHT)
	if Input.is_action_pressed("move_downleft"):
		move_player(Vector2i.DOWN + Vector2i.LEFT)
	if Input.is_action_pressed("move_downright"):
		move_player(Vector2i.DOWN + Vector2i.RIGHT)
	if Input.is_action_pressed("move_wait"):
		move_player(Vector2i.ZERO)

	if Input.is_key_pressed(KEY_Q):
		var player = floor.get_entity_by_id(player_id)
		var action = player.get_actions()[0]
		var command = action.to_command(floor, player, Vector2i(0, 1))
		if command:
			floor.do_action(command)
			$Floor.desynced_from_floor = true
	if Input.is_key_pressed(KEY_W):
		var player = floor.get_entity_by_id(player_id)
		var action = player.get_actions()[1]
		var command = action.to_command(floor, player)
		if command:
			floor.do_action(command)
			$Floor.desynced_from_floor = true

	if delta != 0:
		if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
			goto_mouse()


func move_player(dir: Vector2i):
	if $Floor.desynced_from_floor:
		return

	var player = floor.get_entity_by_id(player_id)
	var action: DirectionAction = floor.get_step_macro_action()
	var command = action.to_command(floor, player, dir)
	if command:
		floor.do_action(command)
		$Floor.desynced_from_floor = true


func goto_mouse():
	if $Floor.desynced_from_floor:
		return

	# HACK: Assumes entire game is on the XZ plane.
	# But this is also kind of expected.
	var mouse = get_viewport().get_mouse_position()
	var origin = get_viewport().get_camera_3d().project_ray_origin(mouse)
	var direction = get_viewport().get_camera_3d().project_ray_normal(mouse)

	var projected_xz: Vector3 = origin + (-origin.y / direction.y) * direction
	var rounded = projected_xz.round()
	var absolute_position = Vector2i(rounded.x, rounded.z)
	
	$Cursor.position = rounded + Vector3.UP * 0.01

	print("absolute position", absolute_position)

	var player = floor.get_entity_by_id(player_id)
	var action: TileAction = floor.get_goto_action()
	var command = action.to_command(floor, player, absolute_position)
	if command:
		floor.do_action(command)
		$Floor.desynced_from_floor = true
