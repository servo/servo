#version {{version}}
// Automatically generated from files in pathfinder/shaders/. Do not edit!












precision highp float;

in vec3 aPosition;

void main(){
    gl_Position = vec4(aPosition, 1.0);
}

