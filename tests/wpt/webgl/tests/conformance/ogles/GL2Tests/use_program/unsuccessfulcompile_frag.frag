
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
uniform float GrainSize;
uniform vec3  DarkColor;
uniform vec3  colorSpread;

varying float lightIntensity; 
varying vec3 Position;

void main (void)
{
    //
    // cheap noise
    //
    vec3 location = Position;

    vec3 floorvec = vec3(floor(Position.x * 10.0), 0.0, floor(Position.z * 10.0));
    vec3 noise = Position * 10.0 - floorvec - 0.5;
    noise *= noise;
    location += noise * 0.12;

    //
    // distance from axis
    //
    float dist = location.x * location.x + location.z * location.z;
    float grain = dist / GrainSize;

    //
    // grain effects as function of distance
    //
    float brightness = fract(grain);
    if (brightness > 0.5) 
        brightness = (1.0 - brightness);
    vec3 color = DarkColor + 0.5 * brightness * (colorSpread);
    
    brightness = fract(grain*7.0);    
    if (brightness > 0.5) 
        brightness = 1.0 - brightness;
    color -= 0.5 * brightness * colorSpread;

    //
    // also as a function of lines parallel to the axis
    //
    brightness = fract(grain*47.0);
    float line = fract(Position.z + Position.x);
    float snap = floor(line * 30.0) * (1.0/30.0);
    if (line < snap + 0.004)
        color -= 0.5 * brightness * colorSpread;

    //
    // apply lighting effects from vertex processor
    //
    color *= lightIntensity;
    color = clamp(color, 0.0, 1.0); 

    gl_FragColor = vec4(color, 0.1)
}
