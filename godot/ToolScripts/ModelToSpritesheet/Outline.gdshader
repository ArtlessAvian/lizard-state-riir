shader_type canvas_item;

void fragment() {
	vec4 yeah = texture(TEXTURE, UV);
	if (yeah.a == 0.0) {
		COLOR = vec4(0, 0, 0, 0);
		COLOR.a += texture(TEXTURE, UV + TEXTURE_PIXEL_SIZE * vec2(0, 1)).a;
		COLOR.a += texture(TEXTURE, UV + TEXTURE_PIXEL_SIZE * vec2(0, -1)).a;
		COLOR.a += texture(TEXTURE, UV + TEXTURE_PIXEL_SIZE * vec2(1, 0)).a;
		COLOR.a += texture(TEXTURE, UV + TEXTURE_PIXEL_SIZE * vec2(-1, 0)).a;
	} else {
		COLOR = yeah;
	}
}