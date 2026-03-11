shader_type spatial;
render_mode cull_front;
render_mode unshaded, world_vertex_coords;

#include "billboardify.gdshaderinc"

uniform vec3 outline_color: source_color = vec3(0, 1.0, 0.5);
uniform ivec2 pixel_shift;

void vertex() {
	vec3 cam_dir_drop_y = normalize(CAMERA_DIRECTION_WORLD * vec3(1.0, 0, 1.0));
	mat3 counterrotation = mat3(
		cross(cam_dir_drop_y, vec3(0, -1, 0)),
		vec3(0, 1, 0),
		cam_dir_drop_y
	);
	
	vec3 dingus = normalize(mat3(MODEL_MATRIX)[2] * vec3(1, 0, 1));
	counterrotation *= inverse(mat3(
		cross(dingus, vec3(0, -1, 0)),
		vec3(0, 1, 0),
		dingus
	));
	
	mat3 discrete = discrete_y_rotations(CAMERA_DIRECTION_WORLD, mat3(MODEL_MATRIX));
	VERTEX = (MODEL_MATRIX * mat4(counterrotation * discrete) * inverse(MODEL_MATRIX) * vec4(VERTEX, 1.0)).xyz;
	
	vec4 clip_space = PROJECTION_MATRIX * VIEW_MATRIX * vec4(VERTEX, 1);
	vec3 yeah = clip_space.xyz / clip_space.w;
	
	vec3 thingy = vec3(vec2(pixel_shift) * 2.0 / VIEWPORT_SIZE, 0);
	vec4 thingy2 = vec4((thingy + yeah) * clip_space.w, clip_space.w);
	VERTEX = (INV_VIEW_MATRIX * INV_PROJECTION_MATRIX * thingy2).xyz;
}

void fragment() {
	ALBEDO = outline_color;
	
	// The depth of a reference point.
	vec4 node_clip = (PROJECTION_MATRIX * vec4(NODE_POSITION_VIEW, 1.0));
	float node_depth = (node_clip.z) / node_clip.w;
	
	vec3 worldspace_vertex = (INV_VIEW_MATRIX * vec4(VERTEX, 1)).xyz;
	vec3 worldspace_dir = mat3(INV_VIEW_MATRIX) * VERTEX;
	vec3 project_y_eqz = worldspace_vertex - ((worldspace_vertex.y - 0.01) / worldspace_dir.y) * worldspace_dir;
	vec4 projected_clip = (PROJECTION_MATRIX * VIEW_MATRIX * vec4(project_y_eqz, 1));
	float projected_depth = projected_clip.z / projected_clip.w;
	
	DEPTH = max(node_depth, projected_depth);
}
