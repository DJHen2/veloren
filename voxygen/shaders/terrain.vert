#version 330 core

#include <globals.glsl>

in uint v_pos_norm;
in uint v_col_light;

layout (std140)
uniform u_locals {
	vec3 model_offs;
};

out vec3 f_pos;
out vec3 f_norm;
out vec3 f_col;
out float f_light;

void main() {
	f_pos = vec3(
		float((v_pos_norm >>  0) & 0x00FFu),
		float((v_pos_norm >>  8) & 0x00FFu),
		float((v_pos_norm >> 16) & 0x1FFFu)
	) + model_offs;

	f_col = vec3(
		float((v_col_light >>  8) & 0xFFu),
		float((v_col_light >> 16) & 0xFFu),
		float((v_col_light >> 24) & 0xFFu)
	) / 255.0;

	uint norm_axis = (v_pos_norm >> 30) & 0x3u;
	float norm_dir = float((v_pos_norm >> 29) & 0x1u) * 2.0 - 1.0;
	if (norm_axis == 0u) {
		f_norm = vec3(1.0, 0.0, 0.0) * norm_dir;
	} else if (norm_axis == 1u) {
		f_norm = vec3(0.0, 1.0, 0.0) * norm_dir;
	} else {
		f_norm = vec3(0.0, 0.0, 1.0) * norm_dir;
	}

	f_light = float(v_col_light & 0xFFu) / 255.0;

	gl_Position =
		proj_mat *
		view_mat *
		vec4(f_pos, 1);
}
