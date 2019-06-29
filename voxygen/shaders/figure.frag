#version 330 core

#include <globals.glsl>
#include <sky.glsl>

in vec3 f_pos;
in vec3 f_norm;
in vec3 f_col;
flat in uint f_bone_idx;

layout (std140)
uniform u_locals {
	mat4 model_mat;
	vec4 model_col;
};

struct BoneData {
	mat4 bone_mat;
};

layout (std140)
uniform u_bones {
	BoneData bones[16];
};

out vec4 tgt_color;

void main() {
	vec3 world_norm = (
		model_mat *
		bones[f_bone_idx].bone_mat *
		vec4(f_norm, 0.0)
	).xyz;

	vec3 light = get_sun_diffuse(world_norm, time_of_day.x);
	vec3 surf_color = model_col.rgb * f_col * 2.0 * light;

	float fog_level = fog(f_pos.xy, focus_pos.xy);
	vec3 fog_color = get_sky_color(normalize(f_pos - cam_pos.xyz), time_of_day.x);
	vec3 color = mix(surf_color, fog_color, fog_level);

	tgt_color = vec4(color, 1.0);
}
