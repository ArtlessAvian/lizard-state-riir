shader_type spatial;

// Makes lines appear pixel perfect from the canonical view angle.
const float TILE_DIMENSIONS_RATIO = 2.0/3.0;
const vec2 EDGE_CUTOFF = vec2(1.0/24.0, 1.0/24.0 / TILE_DIMENSIONS_RATIO);

void fragment() {
	vec2 modelspace_xz = (inverse(MODEL_MATRIX) * INV_VIEW_MATRIX * vec4(VERTEX, 1)).xz;
	
	vec2 elementwise_distance_to_center = abs(modelspace_xz - round(modelspace_xz));
	vec2 elementwise_distance_to_edge = 0.5 - elementwise_distance_to_center;
	vec2 elementwise_cutoff = step(EDGE_CUTOFF, elementwise_distance_to_edge);
	bool edge = min(elementwise_cutoff.x, elementwise_cutoff.y) == 0.0; // safe float comparison because constant from step()
	
	bool checker = (int(round(modelspace_xz.x)) - int(round(modelspace_xz.y))) % 2 == 0;
	vec3 base_color = checker ? vec3(0.2, 0.2, 0.6) : vec3(0.2, 0.2, 0.3);
	
	ALBEDO = base_color * (edge ? 0.3 : 1.0);
}