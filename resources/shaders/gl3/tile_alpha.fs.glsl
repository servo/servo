#version {{version}}
// Automatically generated from files in pathfinder/shaders/. Do not edit!












precision highp float;

uniform sampler2D uStencilTexture;

in vec2 vTexCoord;
in float vBackdrop;
in vec4 vColor;

out vec4 oFragColor;

void main(){
    float coverage = abs(texture(uStencilTexture, vTexCoord). r + vBackdrop);
    oFragColor = vec4(vColor . rgb, vColor . a * coverage);
}

