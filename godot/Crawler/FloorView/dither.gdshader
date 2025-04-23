shader_type spatial;
render_mode unshaded;

uniform float scoot_amount;
global uniform sampler2D fow_texture;

void vertex() {
	VERTEX += (inverse(MODELVIEW_MATRIX) * vec4(0.0, 0.0, scoot_amount, 0.0)).xyz;
}

void fragment() {
	highp vec4 worldspace_zero = vec4(vec3(0), 1);
	highp vec4 clip_space_zero = PROJECTION_MATRIX * VIEW_MATRIX * worldspace_zero;
	highp vec2 offset = FRAGCOORD.xy - clip_space_zero.xy * VIEWPORT_SIZE.xy * 0.5;
	
	offset = floor(offset);
	
	if (int(offset.x) % 6 == 0) {
		if (int(offset.y) % 4 == 0) {
			discard;
		}
	}
	
	// Same as WallShader.
	vec4 worldspace = INV_VIEW_MATRIX * vec4(VERTEX, 1);
	vec3 plane_y_eq_one = worldspace.xyz - CAMERA_DIRECTION_WORLD * ((worldspace.y - 1.0) / CAMERA_DIRECTION_WORLD.y);
	vec2 thingy = plane_y_eq_one.xz / 5.0;
	ALBEDO = texture(fow_texture, thingy).rgb;
}
