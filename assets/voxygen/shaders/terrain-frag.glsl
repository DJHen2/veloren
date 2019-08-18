#version 330 core

#include <globals.glsl>

in vec3 f_pos;
flat in vec3 f_norm;
in vec3 f_col;
in float f_light;

layout (std140)
uniform u_locals {
	vec3 model_offs;
};

out vec4 tgt_color;

#include <sky.glsl>
#include <light.glsl>

void main() {
	vec3 light = get_sun_diffuse(f_norm, time_of_day.x) * f_light + light_at(f_pos, f_norm);
	vec3 surf_color = f_col * light;

	float fog_level = fog(f_pos.xyz, focus_pos.xyz, medium.x);
	vec3 fog_color = get_sky_color(normalize(f_pos - cam_pos.xyz), time_of_day.x);
	vec3 color = mix(surf_color, fog_color, fog_level);

	tgt_color = vec4(color, 1.0);
}
