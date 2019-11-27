#version {{version}}
// Automatically generated from files in pathfinder/shaders/. Do not edit!












#extension GL_GOOGLE_include_directive : enable

precision highp float;












uniform mat4 uTransform;
uniform vec2 uTileSize;
uniform vec2 uStencilTextureSize;

in uvec2 aTessCoord;
in uvec3 aTileOrigin;
in int aBackdrop;
in int aTileIndex;

out vec2 vTexCoord;
out float vBackdrop;
out vec4 vColor;

vec4 getColor();

vec2 computeTileOffset(uint tileIndex, float stencilTextureWidth){
    uint tilesPerRow = uint(stencilTextureWidth / uTileSize . x);
    uvec2 tileOffset = uvec2(tileIndex % tilesPerRow, tileIndex / tilesPerRow);
    return vec2(tileOffset)* uTileSize;
}

void computeVaryings(){
    vec2 origin = vec2(aTileOrigin . xy)+ vec2(aTileOrigin . z & 15u, aTileOrigin . z >> 4u)* 256.0;
    vec2 position =(origin + vec2(aTessCoord))* uTileSize;
    vec2 maskTexCoordOrigin = computeTileOffset(uint(aTileIndex), uStencilTextureSize . x);
    vec2 maskTexCoord = maskTexCoordOrigin + aTessCoord * uTileSize;

    vTexCoord = maskTexCoord / uStencilTextureSize;
    vBackdrop = float(aBackdrop);
    vColor = getColor();
    gl_Position = uTransform * vec4(position, 0.0, 1.0);
}













uniform sampler2D uPaintTexture;
uniform vec2 uPaintTextureSize;

in vec2 aColorTexCoord;

vec4 getColor(){
    return texture(uPaintTexture, aColorTexCoord);
}


void main(){
    computeVaryings();
}

