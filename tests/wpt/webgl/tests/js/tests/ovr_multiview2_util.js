/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

"use strict";

function createTextureWithNearestFiltering(target)
{
    let texture = gl.createTexture();
    gl.bindTexture(target, texture);
    gl.texParameteri(target, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(target, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(target, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(target, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "texture parameter setup should succeed");
    return texture;
}

// Write a transformation matrix to elements of floatArray starting from index.
// The matrix transforms a unit square (-1 to 1) to a rectangle with the width scaleX and the left edge at offsetX.
function setupTranslateAndScaleXMatrix(floatArray, index, scaleX, offsetX)
{
    // x position is transformed according to this equation: scaleX * x0 + translateX = offsetX
    // By substituting x0 with -1 (unit square x value for the left edge), we get the following:
    let translateX = offsetX + scaleX;

    floatArray[index] = scaleX;
    floatArray[index + 1] = 0.0;
    floatArray[index + 2] = 0.0;
    floatArray[index + 3] = 0.0;

    floatArray[index + 4] = 0.0;
    floatArray[index + 5] = 1.0;
    floatArray[index + 6] = 0.0;
    floatArray[index + 7] = 0.0;

    floatArray[index + 8] = 0.0;
    floatArray[index + 9] = 0.0;
    floatArray[index + 10] = 1.0;
    floatArray[index + 11] = 0.0;

    floatArray[index + 12] = translateX;
    floatArray[index + 13] = 0.0;
    floatArray[index + 14] = 0.0;
    floatArray[index + 15] = 1.0;
}

// Check the currently bound read framebuffer with dimensions <width> x <height>.
// The framebuffer should be divided into <strips> equally wide vertical strips, with the one indicated by
// <coloredStripIndex> colored with <expectedStripColor>. The rest of the framebuffer should be colored transparent black.
// A two pixel wide region at each edge of the colored region is left unchecked to allow for some tolerance for rasterization.
function checkVerticalStrip(width, height, strips, coloredStripIndex, expectedStripColor, framebufferDescription)
{
    let colorRegionLeftEdge = (width / strips) * coloredStripIndex;
    let colorRegionRightEdge = (width / strips) * (coloredStripIndex + 1);
    if (coloredStripIndex > 0) {
        wtu.checkCanvasRect(gl, 0, 0, colorRegionLeftEdge - 1, height, [0, 0, 0, 0], 'the left edge of ' + framebufferDescription + ' should be untouched');
    }
    if (coloredStripIndex < strips - 1) {
        wtu.checkCanvasRect(gl, colorRegionRightEdge + 1, 0, width - colorRegionRightEdge - 1, height, [0, 0, 0, 0], 'the right edge of ' + framebufferDescription + ' should be untouched');
    }
    wtu.checkCanvasRect(gl, colorRegionLeftEdge + 1, 0, colorRegionRightEdge - colorRegionLeftEdge - 2, height, expectedStripColor, 'a thin strip in ' + framebufferDescription + ' should be colored ' + expectedStripColor);
}

function getMultiviewPassthroughVertexShader(views) {
    let shaderCode = ['#version 300 es',
    '#extension GL_OVR_multiview2 : require',

    'layout(num_views = $(num_views)) in;',

    'in vec4 a_position;',

    'void main() {',
    '    gl_Position = a_position;',
    '}'].join('\n');
    return wtu.replaceParams(shaderCode, {'num_views': views});
}

// This shader splits the viewport into <views> equally sized vertical strips.
// The input quad defined by "a_position" is transformed to fill a different
// strip in each view.
function getMultiviewOffsetVertexShader(views) {
    let shaderCode = ['#version 300 es',
    '#extension GL_OVR_multiview2 : require',

    'layout(num_views = $(num_views)) in;',

    'in vec4 a_position;',

    'void main() {',
    '    vec4 pos = a_position;',
    "    // Transform the quad to a thin vertical strip that's offset along the x axis according to the view id.",
    '    pos.x = (pos.x * 0.5 + 0.5 + float(gl_ViewID_OVR)) * 2.0 / $(num_views).0 - 1.0;',
    '    gl_Position = pos;',
    '}'].join('\n');
    return wtu.replaceParams(shaderCode, {'num_views': views});
}

// This shader transforms the incoming "a_position" with transforms for each
// view given in the uniform array "transform".
function getMultiviewRealisticUseCaseVertexShader(views) {
    let shaderCode = ['#version 300 es',
    '#extension GL_OVR_multiview2 : require',

    'layout(num_views = $(num_views)) in;',

    'uniform mat4 transform[$(num_views)];',
    'in vec4 a_position;',

    'void main() {',
    "    // Transform the quad with the transformation matrix chosen according to gl_ViewID_OVR.",
    '    vec4 pos = transform[gl_ViewID_OVR] * a_position;',
    '    gl_Position = pos;',
    '}'].join('\n');
    return wtu.replaceParams(shaderCode, {'num_views': views});
}

function getMultiviewColorFragmentShader() {
    return ['#version 300 es',
    '#extension GL_OVR_multiview2 : require',
    'precision highp float;',

    'out vec4 my_FragColor;',

    'void main() {',
    '    uint mask = gl_ViewID_OVR + 1u;',
    '    my_FragColor = vec4(((mask & 4u) != 0u) ? 1.0 : 0.0,',
    '                        ((mask & 2u) != 0u) ? 1.0 : 0.0,',
    '                        ((mask & 1u) != 0u) ? 1.0 : 0.0,',
    '                        1.0);',
    '}'].join('\n');
}

function getMultiviewColorFragmentShaderForDrawBuffers(drawBuffers) {
    let shaderCode = ['#version 300 es',
    '#extension GL_OVR_multiview2 : require',
    'precision highp float;',

    'out vec4 my_FragColor[$(drawBuffers)];',

    'void main() {',
    '    uint mask;'];

    for (let i = 0; i < drawBuffers; ++i) {
        shaderCode.push(wtu.replaceParams('    mask = gl_ViewID_OVR + $(i)u;', {'i': i + 1}));
        shaderCode.push(wtu.replaceParams('    my_FragColor[$(i)] = vec4(((mask & 4u) != 0u) ? 1.0 : 0.0,', {'i': i}));
        shaderCode.push('                           ((mask & 2u) != 0u) ? 1.0 : 0.0,');
        shaderCode.push('                           ((mask & 1u) != 0u) ? 1.0 : 0.0,');
        shaderCode.push('                           1.0);');
    }
    shaderCode.push('}');
    shaderCode = shaderCode.join('\n');
    return wtu.replaceParams(shaderCode, {'drawBuffers' : drawBuffers});
}

function getMultiviewVaryingVertexShader(views) {
    let shaderCode = ['#version 300 es',
    '#extension GL_OVR_multiview2 : require',

    'layout(num_views = $(num_views)) in;',

    'in vec4 a_position;',
    'out float testVarying;',

    'void main() {',
    '    gl_Position = a_position;',
    '    testVarying = float(gl_ViewID_OVR);',
    '}'].join('\n');
    return wtu.replaceParams(shaderCode, {'num_views': views});
}

function getMultiviewVaryingFragmentShader() {
    return ['#version 300 es',
    '#extension GL_OVR_multiview2 : require',
    'precision highp float;',

    'in float testVarying;',
    'out vec4 my_FragColor;',

    'void main() {',
    '    int mask = int(testVarying + 0.1) + 1;',
    '    my_FragColor = vec4(((mask & 4) != 0) ? 1.0 : 0.0,',
    '                        ((mask & 2) != 0) ? 1.0 : 0.0,',
    '                        ((mask & 1) != 0) ? 1.0 : 0.0,',
    '                        1.0);',
    '}'].join('\n');
}

function getMultiviewFlatVaryingVertexShader(views) {
    let shaderCode = ['#version 300 es',
    '#extension GL_OVR_multiview2 : require',

    'layout(num_views = $(num_views)) in;',

    'in vec4 a_position;',
    'flat out int testVarying;',

    'void main() {',
    '    gl_Position = a_position;',
    '    testVarying = int(gl_ViewID_OVR);',
    '}'].join('\n');
    return wtu.replaceParams(shaderCode, {'num_views': views});
}

function getMultiviewFlatVaryingFragmentShader() {
    return ['#version 300 es',
    '#extension GL_OVR_multiview2 : require',
    'precision highp float;',

    'flat in int testVarying;',
    'out vec4 my_FragColor;',

    'void main() {',
    '    int mask = testVarying + 1;',
    '    my_FragColor = vec4(((mask & 4) != 0) ? 1.0 : 0.0,',
    '                        ((mask & 2) != 0) ? 1.0 : 0.0,',
    '                        ((mask & 1) != 0) ? 1.0 : 0.0,',
    '                        1.0);',
    '}'].join('\n');
}

function getMultiviewInstancedVertexShader(views) {
    let shaderCode = ['#version 300 es',
    '#extension GL_OVR_multiview2 : require',

    'layout(num_views = $(num_views)) in;',

    'in vec4 a_position;',
    'out vec4 color;',

    'void main() {',
    '    vec4 pos = a_position;',
    "    // Transform the quad to a thin vertical strip that's offset along the x axis according to the view id and instance id.",
    '    pos.x = (pos.x * 0.5 + 0.5 + float(gl_ViewID_OVR) + float(gl_InstanceID)) * 2.0 / ($(num_views).0 * 2.0) - 1.0;',
    '    int mask = gl_InstanceID + 1;',
    '    color = vec4(((mask & 4) != 0) ? 1.0 : 0.0,',
    '                 ((mask & 2) != 0) ? 1.0 : 0.0,',
    '                 ((mask & 1) != 0) ? 1.0 : 0.0,',
    '                 1.0);',
    '    gl_Position = pos;',
    '}'].join('\n');
    return wtu.replaceParams(shaderCode, {'num_views': views});
}

function getInstanceColorFragmentShader() {
    return ['#version 300 es',
    '#extension GL_OVR_multiview2 : require',
    'precision highp float;',

    'in vec4 color;',
    'out vec4 my_FragColor;',

    'void main() {',
    '    my_FragColor = color;',
    '}'].join('\n');
}

function getExpectedColor(view) {
    var mask = (view + 1);
    return [(mask & 4) ? 255 : 0, (mask & 2) ? 255 : 0, (mask & 1) ? 255 : 0, 255];
}
