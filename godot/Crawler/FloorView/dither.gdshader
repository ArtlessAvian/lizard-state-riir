shader_type spatial;
render_mode unshaded;

void vertex() {
	// scoot towards the camera a little
	VERTEX += (inverse(MODELVIEW_MATRIX) * vec4(0.0, 0.0, 1e-4, 0.0)).xyz;
}

void fragment() {
	highp vec4 zero = vec4(vec3(0), 1);
	highp vec4 clip_space_zero = PROJECTION_MATRIX * VIEW_MATRIX * zero;
	highp vec4 offset = FRAGCOORD - zero;
	
	if (int(offset.x) % 3 == 0) {
		if (int(offset.y) % 2 == 0) {
			discard;
		}
	}
	
	ALBEDO = vec3(0.073);
}
