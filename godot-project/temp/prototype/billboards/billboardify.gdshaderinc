const mat3 id3 = mat3(
	vec3(1, 0, 0),
	vec3(0, 1, 0),
	vec3(0, 0, 1)
);

// IMO this is surprising behavior, but whatever.
const mat4 id4 = mat4(id3);

mat4 discretize_translation(mat4 transformation, int per_unit) {
	vec4 im_origin = transformation * vec4(0, 0, 0, 1);
	
	mat4 rounded_translation = mat4(
		id4[0],
		id4[1],
		id4[2],
		round(im_origin * float(per_unit)) / float(per_unit) - im_origin
	);
	
	return rounded_translation; 
}

const mat3 eighth_rotation = mat3(normalize(vec3(1, 0, -1)), vec3(0, 1, 0), normalize(vec3(1, 0, 1)));
const mat3 quarter_rotation = mat3(vec3(0, 0, -1), vec3(0, 1, 0), vec3(1, 0, 0));

mat3 discrete_y_rotations(vec3 camera_dir, mat3 model_matrix) {
	float yea = dot(normalize(camera_dir.xz), normalize(model_matrix[2].xz));
	if (yea > inversesqrt(2)) {
		return id3;
	} else if (yea < -inversesqrt(2)) {
		return quarter_rotation * quarter_rotation;
	} else if (determinant(mat2(camera_dir.xz, model_matrix[2].xz)) < 0.0) {
		return quarter_rotation;
	} else {
		return quarter_rotation * quarter_rotation * quarter_rotation;
	}
}

mat3 discrete_y_rotation(vec3 desired_z) {
	const mat3 y_axis_eighth_rotation = mat3(normalize(vec3(1, 0, -1)), vec3(0, 1, 0), normalize(vec3(1, 0, 1)));
	
	mat3 current_pow = mat3(vec3(1, 0, 0), vec3(0, 1, 0), vec3(0, 0, 1));
	mat3 best_power = current_pow;
	float best_z = desired_z.z;
	for (int i = 0; i < 8; i++) {
		current_pow *= y_axis_eighth_rotation;
		if ((current_pow * desired_z).z > best_z) {
			best_z = (current_pow * desired_z).z;
			best_power = current_pow;
		}
	}
	
	return best_power;
}

// Projects a point onto a plane perpendicular to the camera, or the XZ plane.
vec3 billboardify(vec3 point, vec3 cam_dir, vec3 plane_point, float eps) {
	float flatten_billboard = dot(cam_dir, plane_point - point);
	// vertex.y should always be positive. cam_dir.y should always be negative.
	float flatten_against_floor = point.y / -cam_dir.y;
	
	return point + (1.0 - eps) * max(flatten_billboard, flatten_against_floor) * cam_dir;
}