#version {{version}}
// Automatically generated from files in pathfinder/shaders/. Do not edit!












precision highp float;

uniform mat4 uOldTransform;
uniform sampler2D uTexture;

in vec2 vTexCoord;

out vec4 oFragColor;

void main(){
    vec4 normTexCoord = uOldTransform * vec4(vTexCoord, 0.0, 1.0);
    vec2 texCoord =((normTexCoord . xy / normTexCoord . w)+ 1.0)* 0.5;
    oFragColor = texture(uTexture, texCoord);
}

