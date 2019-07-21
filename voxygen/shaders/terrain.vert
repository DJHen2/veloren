#version 330 core

#include <globals.glsl>

in uint v_pos_norm;
in uint v_col_light;

layout (std140)
uniform u_locals {
	vec3 model_offs;
};

struct Light {
	vec4 light_pos;
	vec4 light_col;
};

layout (std140)
uniform u_lights {
	Light lights[32];
};

out vec3 f_pos;
flat out uint f_pos_norm;
out vec3 f_col;
out float f_light;

void main() {
	f_pos = vec3(
		float((v_pos_norm >>  0) & 0x00FFu),
		float((v_pos_norm >>  8) & 0x00FFu),
		float((v_pos_norm >> 16) & 0x1FFFu)
	) + model_offs;

	f_pos_norm = v_pos_norm;

	f_col = vec3(
		float((v_col_light >>  8) & 0xFFu),
		float((v_col_light >> 16) & 0xFFu),
		float((v_col_light >> 24) & 0xFFu)
	) / 200.0;

	f_light = float(v_col_light & 0xFFu) / 255.0;

	gl_Position =
		proj_mat *
		view_mat *
		vec4(f_pos, 1);
}
