#version {{version}}
// Automatically generated from files in pathfinder/shaders/. Do not edit!















#extension GL_GOOGLE_include_directive : enable

precision highp float;

uniform sampler2D uSource;
uniform vec2 uSourceSize;
uniform vec4 uFGColor;
uniform vec4 uBGColor;
uniform int uGammaCorrectionEnabled;

in vec2 vTexCoord;

out vec4 oFragColor;














uniform sampler2D uGammaLUT;

float gammaCorrectChannel(float bgColor, float fgColor){
    return texture(uGammaLUT, vec2(fgColor, 1.0 - bgColor)). r;
}


vec3 gammaCorrect(vec3 bgColor, vec3 fgColor){
    return vec3(gammaCorrectChannel(bgColor . r, fgColor . r),
                gammaCorrectChannel(bgColor . g, fgColor . g),
                gammaCorrectChannel(bgColor . b, fgColor . b));
}













uniform vec4 uKernel;



float sample1Tap(float offset);


void sample9Tap(out vec4 outAlphaLeft,
                out float outAlphaCenter,
                out vec4 outAlphaRight,
                float onePixel){
    outAlphaLeft = vec4(uKernel . x > 0.0 ? sample1Tap(- 4.0 * onePixel): 0.0,
                          sample1Tap(- 3.0 * onePixel),
                          sample1Tap(- 2.0 * onePixel),
                          sample1Tap(- 1.0 * onePixel));
    outAlphaCenter = sample1Tap(0.0);
    outAlphaRight = vec4(sample1Tap(1.0 * onePixel),
                          sample1Tap(2.0 * onePixel),
                          sample1Tap(3.0 * onePixel),
                          uKernel . x > 0.0 ? sample1Tap(4.0 * onePixel): 0.0);
}


float convolve7Tap(vec4 alpha0, vec3 alpha1){
    return dot(alpha0, uKernel)+ dot(alpha1, uKernel . zyx);
}



float sample1Tap(float offset){
    return texture(uSource, vec2(vTexCoord . x + offset, vTexCoord . y)). r;
}

void main(){

    vec3 alpha;
    if(uKernel . w == 0.0){
        alpha = texture(uSource, vTexCoord). rrr;
    } else {
        vec4 alphaLeft, alphaRight;
        float alphaCenter;
        sample9Tap(alphaLeft, alphaCenter, alphaRight, 1.0 / uSourceSize . x);

        float r = convolve7Tap(alphaLeft, vec3(alphaCenter, alphaRight . xy));
        float g = convolve7Tap(vec4(alphaLeft . yzw, alphaCenter), alphaRight . xyz);
        float b = convolve7Tap(vec4(alphaLeft . zw, alphaCenter, alphaRight . x), alphaRight . yzw);

        alpha = vec3(r, g, b);
    }


    if(uGammaCorrectionEnabled != 0)
        alpha = gammaCorrect(uBGColor . rgb, alpha);


    oFragColor = vec4(mix(uBGColor . rgb, uFGColor . rgb, alpha), 1.0);
}

