#version {{version}}
// Automatically generated from files in pathfinder/shaders/. Do not edit!












precision highp float;

in ivec2 aPosition;

out vec2 vTexCoord;

void main(){
    vec2 position = vec2(aPosition);
    vTexCoord = position;





    gl_Position = vec4(vec2(position)* 2.0 - 1.0, 0.0, 1.0);
}

