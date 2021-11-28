#version 450 core

layout (location = 0) in vec3 Vertex_Position;   // the position variable has attribute position 0
layout (location = 1) in vec3 Vertex_Color; 
layout (location = 2) in vec3 Vertex_Normal; 

out flat int instance_id;

out vec3 vertex_color; 
out vec3 vertex_normal_ws;
out vec3 vertex_position_ws;

struct InstancedWallData {
    mat4 transform;
    vec4 curve_uv_bbx_minmax;
};

uniform float wall_length;

// shader storage buffer
layout (std430, binding=2) buffer instanced_wall_data
{ 
    InstancedWallData instances[];
};

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;


float random_f(float x) {
    return fract(sin(x*12.9898) * 43758.5453);
}

float fit01(float x, float min, float max) {
    return x * (max-min) + min;
}

float fit(float x, float from_min, float from_max, float to_min, float to_max) {
    // normalize incoming x
    float x_01 = (x - from_min) / (from_max - from_min);
    float x_final = fit01(x_01, to_min, to_max);
    return x_final;
}

vec2 local_to_curve_space(vec2 local, vec4 uv_bbx_bounds) {
    vec2 bbx_min = uv_bbx_bounds.xy;
    vec2 bbx_max = uv_bbx_bounds.zw;

    // Harcoded -0.5 to 0.5 range, because that's the bounds of the brick.glb
    float u = fit(local.x, -0.5, 0.5, bbx_min.x, bbx_max.x);
    float v = fit(local.y, -0.5, 0.5, bbx_min.y, bbx_max.y);

    return vec2(u, v);
}

void main()
{   
    // ws - world space
    // ms - model space (aka single brick)
    // cs - curve space

    float SEED = 112.0;

    mat4 instance_transform = instances[gl_InstanceID].transform;

    vec2 uv_cs = local_to_curve_space(Vertex_Position.xy, instances[gl_InstanceID].curve_uv_bbx_minmax);
    vec4 vertex_ws = model * instance_transform  * vec4(Vertex_Position, 1.0);

    // Bounding Box Y that only takes into account bottom and top of the brick (so rows in-between bricks get same random number)
    float row_bby_ms;
    if (Vertex_Position.y < 0.0) {
        row_bby_ms = -0.5;
    } else {
        row_bby_ms = 0.5;
    }
    float row_bby_cs = local_to_curve_space(vec2(Vertex_Position.x, row_bby_ms), instances[gl_InstanceID].curve_uv_bbx_minmax).y;

    // Add wavey pattern
    vec3 p = vertex_ws.xyz;
    vec3 final_p = p;

    // exclude bottom row
    if (uv_cs.y > 0.1) {

        float r;
        if (uv_cs.y > 1.05) { // if its a top row
            // make it random per brick
            r = float(gl_InstanceID);
            r = random_f(r + SEED);
        } else {
            // otherwise, make it random per row
            r = int(floor(row_bby_cs*100.0));
            r = random_f(r + SEED);
        }

        float freq = fit01(r, 0.01, 0.5) * 10.0;

        float rand_offset = random_f(r*2.0)*1000.0;

        float str = fit01(random_f(r*r+88.0), 0.015, 0.045);

        vec2 vertex_cs = uv_cs * wall_length;
        float sin_wave = sin(vertex_cs.x*freq + rand_offset) * str;

        final_p = vec3(p.x, p.y + sin_wave, p.z);

    }

    // ----------------------------------

    vertex_ws = vec4(final_p, 1.0);

    gl_Position = projection * view * vertex_ws;
    instance_id = gl_InstanceID;
    vertex_color = Vertex_Color;
    vertex_position_ws = vertex_ws.xyz;
    vertex_normal_ws = (instance_transform * vec4(Vertex_Normal, 0.0)).xyz;
} 
