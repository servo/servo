#version {{version}}
// Automatically generated from files in pathfinder/shaders/. Do not edit!












precision highp float;

uniform mat4 uNewTransform;

in ivec2 aPosition;

out vec2 vTexCoord;

void main(){
    vec2 position = vec2(aPosition);
    vTexCoord = position;





    gl_Position = uNewTransform * vec4(position, 0.0, 1.0);
}

