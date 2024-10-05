'use strict';

const trivialVsSrc = `
void main()
{
    gl_Position = vec4(0,0,0,1);
}
`;
const trivialFsSrc = `
void main()
{
    gl_FragColor = vec4(0,1,0,1);
}
`;
const trivialVsMrtSrc100 = `
void main()
{
    gl_Position = vec4(0,0,0,1);
}
`;
const trivialFsMrtSrc100 = `
#extension GL_EXT_draw_buffers : require
precision mediump float;
void main()
{
    gl_FragData[0] = vec4(1, 0, 0, 1);
    gl_FragData[1] = vec4(0, 1, 0, 1);
}
`;
const trivialVsMrtSrc300 = `#version 300 es
void main()
{
    gl_Position = vec4(0,0,0,1);
}
`;
const trivialFsMrtSrc300 = `#version 300 es
precision mediump float;
layout(location = 0) out vec4 o_color0;
layout(location = 1) out vec4 o_color1;
void main()
{
    o_color0 = vec4(1, 0, 0, 1);
    o_color1 = vec4(0, 1, 0, 1);
}
`;

function testExtFloatBlend(internalFormat) {
    const shouldBlend = gl.getSupportedExtensions().indexOf('EXT_float_blend') != -1;

    const prog = wtu.setupProgram(gl, [trivialVsSrc, trivialFsSrc]);
    gl.useProgram(prog);

    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D, 0, internalFormat, 1, 1, 0, gl.RGBA, gl.FLOAT, null);

    const fb = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
    shouldBe('gl.checkFramebufferStatus(gl.FRAMEBUFFER)', 'gl.FRAMEBUFFER_COMPLETE');

    gl.disable(gl.BLEND);
    gl.drawArrays(gl.POINTS, 0, 1);
    wtu.glErrorShouldBe(gl, 0, 'Float32 draw target without blending');

    gl.enable(gl.BLEND);
    gl.drawArrays(gl.POINTS, 0, 1);
    wtu.glErrorShouldBe(gl, shouldBlend ? 0 : gl.INVALID_OPERATION,
                        'Float32 blending is ' + (shouldBlend ? '' : 'not ') + 'allowed ');

    gl.deleteFramebuffer(fb);
    gl.deleteTexture(tex);
}

function testExtFloatBlendMRTImpl(version, internalFormat, shaders, attachments, drawBuffers) {
    const shouldBlend = gl.getSupportedExtensions().indexOf('EXT_float_blend') != -1;

    const prog = wtu.setupProgram(gl, shaders);
    gl.useProgram(prog);

    const tex1 = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex1);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);

    const tex2 = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex2);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);

    const texF1 = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, texF1);
    gl.texImage2D(gl.TEXTURE_2D, 0, internalFormat, 1, 1, 0, gl.RGBA, gl.FLOAT, null);

    const texF2 = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, texF2);
    gl.texImage2D(gl.TEXTURE_2D, 0, internalFormat, 1, 1, 0, gl.RGBA, gl.FLOAT, null);

    const fb = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, attachments[0], gl.TEXTURE_2D, tex1, 0);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, attachments[1], gl.TEXTURE_2D, tex2, 0);
    shouldBe('gl.checkFramebufferStatus(gl.FRAMEBUFFER)', 'gl.FRAMEBUFFER_COMPLETE');

    drawBuffers(attachments);

    gl.enable(gl.BLEND);

    gl.drawArrays(gl.POINTS, 0, 1);
    wtu.glErrorShouldBe(gl, 0, 'No Float32 color attachment');

    if (version < 2) {
        // EXT_draw_buffers require all color buffers having the same number of bitplanes
        gl.framebufferTexture2D(gl.FRAMEBUFFER, attachments[0], gl.TEXTURE_2D, texF1, 0);
        gl.framebufferTexture2D(gl.FRAMEBUFFER, attachments[1], gl.TEXTURE_2D, texF2, 0);
        shouldBe('gl.checkFramebufferStatus(gl.FRAMEBUFFER)', 'gl.FRAMEBUFFER_COMPLETE');
        gl.drawArrays(gl.POINTS, 0, 1);
        wtu.glErrorShouldBe(gl, shouldBlend ? 0 : gl.INVALID_OPERATION,
                            'Float32 blending is ' + (shouldBlend ? '' : 'not ') + 'allowed ');
    } else {
        gl.framebufferTexture2D(gl.FRAMEBUFFER, attachments[0], gl.TEXTURE_2D, texF1, 0);
        shouldBe('gl.checkFramebufferStatus(gl.FRAMEBUFFER)', 'gl.FRAMEBUFFER_COMPLETE');
        gl.drawArrays(gl.POINTS, 0, 1);
        wtu.glErrorShouldBe(gl, shouldBlend ? 0 : gl.INVALID_OPERATION,
                            'Float32 blending is ' + (shouldBlend ? '' : 'not ') + 'allowed ');

        gl.framebufferTexture2D(gl.FRAMEBUFFER, attachments[1], gl.TEXTURE_2D, texF2, 0);
        shouldBe('gl.checkFramebufferStatus(gl.FRAMEBUFFER)', 'gl.FRAMEBUFFER_COMPLETE');
        gl.drawArrays(gl.POINTS, 0, 1);
        wtu.glErrorShouldBe(gl, shouldBlend ? 0 : gl.INVALID_OPERATION,
                            'Float32 blending is ' + (shouldBlend ? '' : 'not ') + 'allowed ');

        gl.framebufferTexture2D(gl.FRAMEBUFFER, attachments[0], gl.TEXTURE_2D, tex1, 0);
        shouldBe('gl.checkFramebufferStatus(gl.FRAMEBUFFER)', 'gl.FRAMEBUFFER_COMPLETE');
        gl.drawArrays(gl.POINTS, 0, 1);
        wtu.glErrorShouldBe(gl, shouldBlend ? 0 : gl.INVALID_OPERATION,
                            'Float32 blending is ' + (shouldBlend ? '' : 'not ') + 'allowed ');

        drawBuffers([attachments[0]]);
        shouldBe('gl.checkFramebufferStatus(gl.FRAMEBUFFER)', 'gl.FRAMEBUFFER_COMPLETE');

        gl.drawArrays(gl.POINTS, 0, 1);
        wtu.glErrorShouldBe(gl, 0, 'Float32 color attachment draw buffer is not enabled');

        drawBuffers(attachments);
        gl.framebufferTexture2D(gl.FRAMEBUFFER, attachments[1], gl.TEXTURE_2D, tex2, 0);
        shouldBe('gl.checkFramebufferStatus(gl.FRAMEBUFFER)', 'gl.FRAMEBUFFER_COMPLETE');

        gl.drawArrays(gl.POINTS, 0, 1);
        wtu.glErrorShouldBe(gl, 0, 'No Float32 color attachment');
    }

    gl.deleteFramebuffer(fb);
    gl.deleteTexture(tex1);
    gl.deleteTexture(tex2);
    gl.deleteTexture(texF1);
    gl.deleteTexture(texF2);
}

function testExtFloatBlendMRT(version, drawBuffersExt) {
    if (version < 2) {
        if (!drawBuffersExt) return;
        testExtFloatBlendMRTImpl(
            version,
            gl.RGBA,
            [trivialVsMrtSrc100, trivialFsMrtSrc100],
            [drawBuffersExt.COLOR_ATTACHMENT0_WEBGL, drawBuffersExt.COLOR_ATTACHMENT1_WEBGL],
            drawBuffersExt.drawBuffersWEBGL.bind(drawBuffersExt)
        );
    } else {
        testExtFloatBlendMRTImpl(
            version,
            gl.RGBA32F,
            [trivialVsMrtSrc300, trivialFsMrtSrc300],
            [gl.COLOR_ATTACHMENT0, gl.COLOR_ATTACHMENT1],
            gl.drawBuffers.bind(gl)
        );
    }
}

function testExtFloatBlendNonFloat32TypeImpl(internalFormat, formats) {
    const shouldBlend = gl.getSupportedExtensions().indexOf('EXT_float_blend') != -1;

    const prog = wtu.setupProgram(gl, [trivialVsSrc, trivialFsSrc]);
    gl.useProgram(prog);

    gl.enable(gl.BLEND);

    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D, 0, internalFormat, 1, 1, 0, gl.RGBA, gl.FLOAT, null);

    const fb = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
    shouldBe('gl.checkFramebufferStatus(gl.FRAMEBUFFER)', 'gl.FRAMEBUFFER_COMPLETE');

    gl.drawArrays(gl.POINTS, 0, 1);
    wtu.glErrorShouldBe(gl, shouldBlend ? 0 : gl.INVALID_OPERATION,
                        'Float32 blending is ' + (shouldBlend ? '' : 'not ') + 'allowed ');

    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
    gl.drawArrays(gl.POINTS, 0, 1);
    wtu.glErrorShouldBe(gl, 0, 'UNSIGNED_BYTE should blend anyway');

    for (let i = 0, len = formats.length; i < len; i++) {
        gl.texImage2D(gl.TEXTURE_2D, 0, formats[i][0], 1, 1, 0, formats[i][1], formats[i][2], null);
        gl.drawArrays(gl.POINTS, 0, 1);
        wtu.glErrorShouldBe(gl, 0, 'Any other float type which is not 32-bit-Float should blend anyway');
    }

    gl.deleteFramebuffer(fb);
    gl.deleteTexture(tex);
}

function testExtFloatBlendNonFloat32Type(version, oesTextureHalfFloat) {
    if (version < 2) {
        if (!oesTextureHalfFloat) return;
        const formats = [
            [gl.RGBA, gl.RGBA, oesTextureHalfFloat.HALF_FLOAT_OES]
        ];
        testExtFloatBlendNonFloat32TypeImpl(gl.RGBA, formats);
    } else {
        const formats = [
            [gl.RGBA16F, gl.RGBA, gl.HALF_FLOAT],
            [gl.RGBA16F, gl.RGBA, gl.FLOAT],
            [gl.RG16F, gl.RG, gl.FLOAT],
            [gl.R16F, gl.RED, gl.FLOAT],
            [gl.R11F_G11F_B10F, gl.RGB, gl.FLOAT]
        ];
        testExtFloatBlendNonFloat32TypeImpl(gl.RGBA32F, formats);
    }
}

/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/
