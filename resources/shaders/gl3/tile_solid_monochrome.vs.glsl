#version {{version}}
// Automatically generated from files in pathfinder/shaders/. Do not edit!












#extension GL_GOOGLE_include_directive : enable

precision highp float;












uniform mat4 uTransform;
uniform vec2 uTileSize;

in uvec2 aTessCoord;
in ivec2 aTileOrigin;

out vec4 vColor;

vec4 getColor();

void computeVaryings(){
    vec2 position = vec2(aTileOrigin + ivec2(aTessCoord))* uTileSize;
    vColor = getColor();
    gl_Position = uTransform * vec4(position, 0.0, 1.0);
}












uniform vec4 uColor;

vec4 getColor(){
    return uColor;
}


void main(){
    computeVaryings();
}

