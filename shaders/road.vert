#version 450 core

layout (location = 0) in vec3 Vertex_Position;   // the position variable has attribute position 0
layout (location = 1) in vec3 Vertex_Color; // the color variable has attribute position 1
layout (location = 2) in vec2 Vertex_UV;
// Custom
layout (location = 3) in vec4 bbx_bounds;
//layout (location = 4) in int prim_id; //TODO: why it doesnt work :(
  
out vec3 ourColor; // output a color to the fragment shader
out vec2 TexCoord;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

uniform sampler2D ourTexture;

float sample_texture_ws(vec2 pos_ws) {
    vec2 texture_uv = (pos_ws / 20.0 + 0.5);
    return texture(ourTexture, texture_uv).x;
}

float random_f(float x) {
    return fract(sin(x*12.9898) * 43758.5453);
}

float fit01(float x, float min, float max) {
    return x * (max-min) + min;
}

void main()
{   
    vec3 pos_ws = Vertex_Position;
    vec2 bbx_min = bbx_bounds.xy;
    vec2 bbx_max = bbx_bounds.zw;



    // sample corner of the bbx of this pebble
    float avg_value = 0.0;

    // LEFT BOTTOM
    avg_value += sample_texture_ws(bbx_min);
    // RIGHT BOTTOM
    avg_value += sample_texture_ws(vec2(bbx_max.x, bbx_min.y));
    // LEFT TOP
    avg_value += sample_texture_ws(vec2(bbx_min.x, bbx_max.y));
    // RIGHT TOP
    avg_value += sample_texture_ws(bbx_max);

    avg_value /= 4.0;
    
    float seed = bbx_min.x+bbx_min.y+bbx_max.x+bbx_max.y;
    float threshold = random_f(seed);
    threshold = fit01(threshold, 0.2, 0.4);

    if (avg_value > threshold) {
        pos_ws.y = 0.01;
    } else {
        pos_ws.y = -1;
    }

    //pos_ws.y = 0.01;
    //pos_ws.y = float(prim_id) / 8000.0 * 0.00001;


    // OUT ----------------------------
    gl_Position = projection * view * vec4(pos_ws, 1.0);
    ourColor = vec3(0.3); // set ourColor to the input color we got from the vertex data
    TexCoord = Vertex_UV;
} 