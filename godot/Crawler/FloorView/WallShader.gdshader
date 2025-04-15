shader_type spatial;
render_mode unshaded, cull_disabled;

global uniform sampler2D wall_texture;
global uniform sampler2D oob_texture;

void fragment() {
	vec4 worldspace = INV_VIEW_MATRIX * vec4(VERTEX, 1);
	float yeah = worldspace.x + 0.5 * worldspace.z;
	
	vec3 plane_y_eq_one = worldspace.xyz - CAMERA_DIRECTION_WORLD * ((worldspace.y - 1.0) / CAMERA_DIRECTION_WORLD.y);
	vec2 thingy = plane_y_eq_one.xz / 5.0;
	
	ALBEDO = float(FRONT_FACING) * texture(wall_texture, vec2(yeah, -worldspace.y)).rgb;
	ALBEDO += float(!FRONT_FACING) * texture(oob_texture, thingy).rgb;
}