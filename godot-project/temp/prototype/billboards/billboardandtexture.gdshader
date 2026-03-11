shader_type spatial;
render_mode unshaded, world_vertex_coords, cull_back;

#include "billboardify.gdshaderinc"

uniform sampler2D texture_albedo : source_color, filter_nearest;

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
	//VERTEX = (discretize_translation(VIEW_MATRIX, 24) * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec4 tex = texture(texture_albedo, UV);
	//if (length(tex.rgb) < 0.4) {
		//ALBEDO = vec3(1.0, 0, 1.0);
	//} else
	if (tex.a == 0.0) {
		ALBEDO = vec3(1.0, 0, 1.0);
	} else {
		ALBEDO = tex.rgb;
	}
	
	// The depth of a reference point.
	vec4 node_clip = (PROJECTION_MATRIX * vec4(NODE_POSITION_VIEW, 1.0));
	float node_depth = max(node_clip.z / node_clip.w, mix(node_clip.z / node_clip.w, FRAGCOORD.z, 0.01));
	
	vec3 worldspace_vertex = (INV_VIEW_MATRIX * vec4(VERTEX, 1)).xyz;
	vec3 worldspace_dir = mat3(INV_VIEW_MATRIX) * VERTEX;
	vec3 project_y_eqz = worldspace_vertex - ((worldspace_vertex.y - 0.02) / worldspace_dir.y) * worldspace_dir;
	vec4 projected_clip = (PROJECTION_MATRIX * VIEW_MATRIX * vec4(project_y_eqz, 1));
	float projected_depth = projected_clip.z / projected_clip.w;
	
	DEPTH = max(node_depth, projected_depth);
}
