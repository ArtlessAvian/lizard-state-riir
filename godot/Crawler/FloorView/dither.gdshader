shader_type spatial;
render_mode unshaded;

void vertex() {
	// Called for every vertex the material is visible on.
}

void fragment() {
	if (int(FRAGCOORD.x) % 6 == 0) {
		if (int(FRAGCOORD.y) % 4 == 0) {
			discard;
		}
	}
	ALBEDO = vec3(0.073);
}
