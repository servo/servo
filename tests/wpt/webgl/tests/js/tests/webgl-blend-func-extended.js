"use strict";
description("This test verifies the functionality of the WEBGL_blend_func_extended extension, if it is available.");

debug("");

var wtu = WebGLTestUtils;
var gl = wtu.create3DContext("c", undefined, contextVersion);
var ext;

function runTestNoExtension() {
    debug("");
    debug("Testing getParameter without the extension");
    shouldBeNull("gl.getParameter(0x88FC /* MAX_DUAL_SOURCE_DRAW_BUFFERS_WEBGL */)");
    wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, "parameter unknown");
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");

    if (contextVersion == 1) {
        debug("");
        debug("Testing SRC_ALPHA_SATURATE without the extension");

        gl.blendFunc(gl.ONE, gl.SRC_ALPHA_SATURATE);
        wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, "SRC_ALPHA_SATURATE not accepted as blendFunc dfactor");
        gl.blendFuncSeparate(gl.ONE, gl.SRC_ALPHA_SATURATE, gl.ONE, gl.ONE);
        wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, "SRC_ALPHA_SATURATE not accepted as blendFuncSeparate dstRGB");
        gl.blendFuncSeparate(gl.ONE, gl.ONE, gl.ONE, gl.SRC_ALPHA_SATURATE);
        wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, "SRC_ALPHA_SATURATE not accepted as blendFuncSeparate dstAlpha");
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
    }

    debug("");
    debug("Testing SRC1 blend funcs without the extension");

    const extFuncs = {
        SRC1_COLOR_WEBGL: 0x88F9,
        SRC1_ALPHA_WEBGL: 0x8589,
        ONE_MINUS_SRC1_COLOR_WEBGL: 0x88FA,
        ONE_MINUS_SRC1_ALPHA_WEBGL: 0x88FB
    };

    for (const func in extFuncs) {
        gl.blendFunc(extFuncs[func], gl.ONE);
        wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, `${func} not accepted as blendFunc sfactor`);
        gl.blendFunc(gl.ONE, extFuncs[func]);
        wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, `${func} not accepted as blendFunc dfactor`);
        gl.blendFuncSeparate(extFuncs[func], gl.ONE, gl.ONE, gl.ONE);
        wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, `${func} not accepted as blendFuncSeparate srcRGB`);
        gl.blendFuncSeparate(gl.ONE, extFuncs[func], gl.ONE, gl.ONE);
        wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, `${func} not accepted as blendFuncSeparate dstRGB`);
        gl.blendFuncSeparate(gl.ONE, gl.ONE, extFuncs[func], gl.ONE);
        wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, `${func} not accepted as blendFuncSeparate srcAlpha`);
        gl.blendFuncSeparate(gl.ONE, gl.ONE, gl.ONE, extFuncs[func]);
        wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, `${func} not accepted as blendFuncSeparate dstAlpha`);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
    }

    const dbi = gl.getExtension("OES_draw_buffers_indexed");
    if (!dbi) return;

    debug("");
    debug("Testing indexed SRC1 blend funcs without the extension");
    for (const func in extFuncs) {
        dbi.blendFunciOES(0, extFuncs[func], gl.ONE);
        wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, `${func} not accepted as blendFunciOES src`);
        dbi.blendFunciOES(0, gl.ONE, extFuncs[func]);
        wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, `${func} not accepted as blendFunciOES dst`);
        dbi.blendFuncSeparateiOES(0, extFuncs[func], gl.ONE, gl.ONE, gl.ONE);
        wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, `${func} not accepted as blendFuncSeparateiOES srcRGB`);
        dbi.blendFuncSeparateiOES(0, gl.ONE, extFuncs[func], gl.ONE, gl.ONE);
        wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, `${func} not accepted as blendFuncSeparateiOES dstRGB`);
        dbi.blendFuncSeparateiOES(0, gl.ONE, gl.ONE, extFuncs[func], gl.ONE);
        wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, `${func} not accepted as blendFuncSeparateiOES srcAlpha`);
        dbi.blendFuncSeparateiOES(0, gl.ONE, gl.ONE, gl.ONE, extFuncs[func]);
        wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, `${func} not accepted as blendFuncSeparateiOES dstAlpha`);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
    }
}

function runEnumTests() {
    debug("");
    debug("Testing enums");
    shouldBe("ext.SRC1_COLOR_WEBGL", "0x88F9");
    shouldBe("ext.SRC1_ALPHA_WEBGL", "0x8589");
    shouldBe("ext.ONE_MINUS_SRC1_COLOR_WEBGL", "0x88FA");
    shouldBe("ext.ONE_MINUS_SRC1_ALPHA_WEBGL", "0x88FB");
    shouldBe("ext.MAX_DUAL_SOURCE_DRAW_BUFFERS_WEBGL", "0x88FC");
}

function runQueryTests() {
    debug("");
    debug("Testing getParameter");
    shouldBeGreaterThanOrEqual("gl.getParameter(ext.MAX_DUAL_SOURCE_DRAW_BUFFERS_WEBGL)", "1");
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");

    if (contextVersion == 1) {
        debug("");
        debug("Testing SRC_ALPHA_SATURATE with the extension");

        gl.blendFunc(gl.ONE, gl.SRC_ALPHA_SATURATE);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "SRC_ALPHA_SATURATE accepted as blendFunc dfactor");
        shouldBe("gl.getParameter(gl.BLEND_DST_RGB)", "gl.SRC_ALPHA_SATURATE");
        shouldBe("gl.getParameter(gl.BLEND_DST_ALPHA)", "gl.SRC_ALPHA_SATURATE");
        gl.blendFuncSeparate(gl.ONE, gl.SRC_ALPHA_SATURATE, gl.ONE, gl.ONE);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "SRC_ALPHA_SATURATE accepted as blendFuncSeparate dstRGB");
        shouldBe("gl.getParameter(gl.BLEND_DST_RGB)", "gl.SRC_ALPHA_SATURATE");
        gl.blendFuncSeparate(gl.ONE, gl.ONE, gl.ONE, gl.SRC_ALPHA_SATURATE);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "SRC_ALPHA_SATURATE accepted as blendFuncSeparate dstAlpha");
        shouldBe("gl.getParameter(gl.BLEND_DST_ALPHA)", "gl.SRC_ALPHA_SATURATE");
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
    }

    const extFuncs = [
        "SRC1_COLOR_WEBGL",
        "SRC1_ALPHA_WEBGL",
        "ONE_MINUS_SRC1_COLOR_WEBGL",
        "ONE_MINUS_SRC1_ALPHA_WEBGL"
    ];

    debug("");
    debug("Testing blend state updates with SRC1 blend funcs");
    for (const func of extFuncs) {
        gl.blendFunc(ext[func], gl.ONE);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, `${func} accepted as blendFunc sfactor`);
        shouldBe("gl.getParameter(gl.BLEND_SRC_RGB)", `ext.${func}`);
        shouldBe("gl.getParameter(gl.BLEND_SRC_ALPHA)", `ext.${func}`);
        gl.blendFunc(gl.ONE, ext[func]);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, `${func} accepted as blendFunc dfactor`);
        shouldBe("gl.getParameter(gl.BLEND_DST_RGB)", `ext.${func}`);
        shouldBe("gl.getParameter(gl.BLEND_DST_ALPHA)", `ext.${func}`);
        gl.blendFuncSeparate(ext[func], gl.ONE, gl.ONE, gl.ONE);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, `${func} accepted as blendFuncSeparate srcRGB`);
        shouldBe("gl.getParameter(gl.BLEND_SRC_RGB)", `ext.${func}`);
        gl.blendFuncSeparate(gl.ONE, ext[func], gl.ONE, gl.ONE);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, `${func} accepted as blendFuncSeparate dstRGB`);
        shouldBe("gl.getParameter(gl.BLEND_DST_RGB)", `ext.${func}`);
        gl.blendFuncSeparate(gl.ONE, gl.ONE, ext[func], gl.ONE);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, `${func} accepted as blendFuncSeparate srcAlpha`);
        shouldBe("gl.getParameter(gl.BLEND_SRC_ALPHA)", `ext.${func}`);
        gl.blendFuncSeparate(gl.ONE, gl.ONE, gl.ONE, ext[func]);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, `${func} accepted as blendFuncSeparate dstAlpha`);
        shouldBe("gl.getParameter(gl.BLEND_DST_ALPHA)", `ext.${func}`);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
    }

    const dbi = gl.getExtension("OES_draw_buffers_indexed");
    if (!dbi) return;

    debug("");
    debug("Testing indexed blend state updates with SRC1 blend funcs");
    for (const func of extFuncs) {
        dbi.blendFunciOES(0, ext[func], gl.ONE);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, `${func} accepted as blendFunciOES src`);
        shouldBe("gl.getIndexedParameter(gl.BLEND_SRC_RGB, 0)", `ext.${func}`);
        shouldBe("gl.getIndexedParameter(gl.BLEND_SRC_ALPHA, 0)", `ext.${func}`);
        dbi.blendFunciOES(0, gl.ONE, ext[func]);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, `${func} accepted as blendFunciOES dst`);
        shouldBe("gl.getIndexedParameter(gl.BLEND_DST_RGB, 0)", `ext.${func}`);
        shouldBe("gl.getIndexedParameter(gl.BLEND_DST_ALPHA, 0)", `ext.${func}`);
        dbi.blendFuncSeparateiOES(0, ext[func], gl.ONE, gl.ONE, gl.ONE);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, `${func} not accepted as blendFuncSeparateiOES srcRGB`);
        shouldBe("gl.getIndexedParameter(gl.BLEND_SRC_RGB, 0)", `ext.${func}`);
        dbi.blendFuncSeparateiOES(0, gl.ONE, ext[func], gl.ONE, gl.ONE);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, `${func} not accepted as blendFuncSeparateiOES dstRGB`);
        shouldBe("gl.getIndexedParameter(gl.BLEND_DST_RGB, 0)", `ext.${func}`);
        dbi.blendFuncSeparateiOES(0, gl.ONE, gl.ONE, ext[func], gl.ONE);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, `${func} not accepted as blendFuncSeparateiOES srcAlpha`);
        shouldBe("gl.getIndexedParameter(gl.BLEND_SRC_ALPHA, 0)", `ext.${func}`);
        dbi.blendFuncSeparateiOES(0, gl.ONE, gl.ONE, gl.ONE, ext[func]);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, `${func} not accepted as blendFuncSeparateiOES dstAlpha`);
        shouldBe("gl.getIndexedParameter(gl.BLEND_DST_ALPHA, 0)", `ext.${func}`);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
    }
}

function runShaderTests(extensionEnabled) {
    debug("");
    debug("Testing various shader compiles with extension " + (extensionEnabled ? "enabled" : "disabled"));

    const shaderSets = [];

    const macro100 = `precision mediump float;
        void main() {
        #ifdef GL_EXT_blend_func_extended
            gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
        #else
            #error no GL_EXT_blend_func_extended;
        #endif
        }`;
    const macro300 = `#version 300 es
        out mediump vec4 my_FragColor;
        void main() {
        #ifdef GL_EXT_blend_func_extended
            my_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
        #else
            #error no GL_EXT_blend_func_extended;
        #endif
        }`;
    shaderSets.push([wtu.simpleVertexShader, macro100]);
    if (contextVersion == 2) {
        shaderSets.push([wtu.simpleVertexShaderESSL300, macro300]);
    }

    for (const shaders of shaderSets) {
        // Expect the macro shader to succeed ONLY if enabled
        if (wtu.setupProgram(gl, shaders)) {
            if (extensionEnabled) {
                testPassed("Macro defined in shaders when extension is enabled");
            } else {
                testFailed("Macro defined in shaders when extension is disabled");
            }
        } else {
            if (extensionEnabled) {
                testFailed("Macro not defined in shaders when extension is enabled");
            } else {
                testPassed("Macro not defined in shaders when extension is disabled");
            }
        }
    }

    shaderSets.length = 0;

    const missing100 = `
        void main() {
            gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
            gl_SecondaryFragColorEXT = vec4(0.0, 1.0, 0.0, 1.0);
        }`;
    shaderSets.push([wtu.simpleVertexShader, missing100]);

    const missing300 = `#version 300 es
        layout(location = 0)            out mediump vec4 oColor0;
        layout(location = 0, index = 1) out mediump vec4 oColor1;
        void main() {
            oColor0 = vec4(1.0, 0.0, 0.0, 1.0);
            oColor1 = vec4(0.0, 1.0, 0.0, 1.0);
        }`;
    if (contextVersion == 2) {
        shaderSets.push([wtu.simpleVertexShaderESSL300, missing300]);
    }

    // Always expect the shader missing the #extension pragma to fail (whether enabled or not)
    for (const shaders of shaderSets) {
        if (wtu.setupProgram(gl, shaders)) {
            testFailed("Secondary fragment output allowed without #extension pragma");
        } else {
            testPassed("Secondary fragment output disallowed without #extension pragma");
        }
    }

    shaderSets.length = 0;

    const valid100 = `#extension GL_EXT_blend_func_extended : enable
        void main() {
            gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
            gl_SecondaryFragColorEXT = vec4(0.0, 1.0, 0.0, 1.0);
        }`;
    shaderSets.push([wtu.simpleVertexShader, valid100]);

    const valid300 = `#version 300 es
        #extension GL_EXT_blend_func_extended : enable
        layout(location = 0)            out mediump vec4 oColor0;
        layout(location = 0, index = 1) out mediump vec4 oColor1;
        void main() {
            oColor0 = vec4(1.0, 0.0, 0.0, 1.0);
            oColor1 = vec4(0.0, 1.0, 0.0, 1.0);
        }`;
    if (contextVersion == 2) {
        shaderSets.push([wtu.simpleVertexShaderESSL300, valid300]);
    }

    // Try to compile a shader using a secondary fragment output that should only succeed if enabled
    for (const shaders of shaderSets) {
        if (wtu.setupProgram(gl, shaders)) {
            if (extensionEnabled) {
                testPassed("Secondary fragment output compiled successfully when extension enabled");
            } else {
                testFailed("Secondary fragment output compiled successfully when extension disabled");
            }
        } else {
            if (extensionEnabled) {
                testFailed("Secondary fragment output failed to compile when extension enabled");
            } else {
                testPassed("Secondary fragment output failed to compile when extension disabled");
            }
        }
    }

    // ESSL 3.00: Testing that multiple outputs require explicit locations
    if (contextVersion == 2) {
        const locations300 = `#version 300 es
            #extension GL_EXT_blend_func_extended : enable
            out mediump vec4 color0;
            out mediump vec4 color1;
            void main() {
                color0 = vec4(1.0, 0.0, 0.0, 1.0);
                color1 = vec4(0.0, 1.0, 0.0, 1.0);
            }`;
        if (wtu.setupProgram(gl, [wtu.simpleVertexShaderESSL300, locations300])) {
            testFailed("Multiple fragment outputs compiled successfully without explicit locations");
        } else {
            testPassed("Multiple fragment outputs failed to compile without explicit locations");
        }
    }
}

function runMissingOutputsTests() {
    debug("");
    debug("Test draw calls with missing fragment outputs");

    wtu.setupUnitQuad(gl);
    gl.blendFunc(gl.ONE, ext.SRC1_COLOR_WEBGL);

    for (const enabled of [false, true]) {
        if (enabled) {
            gl.enable(gl.BLEND);
        } else {
            gl.disable(gl.BLEND);
        }

        for (const maskedOut of [false, true]) {
            gl.colorMask(!maskedOut, false, false, false);

            const label = `Dual-source blending ${enabled ? "ENABLED" : "DISABLED"}, ` +
                          `missing fragment outputs, and ` +
                          `${maskedOut ? "" : "NOT "}all color channels masked out`;
            debug(`ESSL 1.00: ${label}`);

            {
                const none = "void main() {}";
                wtu.setupProgram(gl, [wtu.simpleVertexShader, none]);
                wtu.drawUnitQuad(gl);
                wtu.glErrorShouldBe(gl, maskedOut ? gl.NO_ERROR : gl.INVALID_OPERATION,
                                    "no fragment outputs");
            }

            {
                const fragColor = `
                    void main() {
                        gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                    }`;
                wtu.setupProgram(gl, [wtu.simpleVertexShader, fragColor]);
                wtu.drawUnitQuad(gl);
                wtu.glErrorShouldBe(gl, (!enabled || maskedOut) ? gl.NO_ERROR : gl.INVALID_OPERATION,
                                    "only gl_FragColor");
            }

            {
                const secondaryFragColor = `#extension GL_EXT_blend_func_extended : enable
                    void main() {
                        gl_SecondaryFragColorEXT = vec4(0.0, 1.0, 0.0, 1.0);
                    }`;
                wtu.setupProgram(gl, [wtu.simpleVertexShader, secondaryFragColor]);
                wtu.drawUnitQuad(gl);
                wtu.glErrorShouldBe(gl, maskedOut ? gl.NO_ERROR : gl.INVALID_OPERATION,
                                    "only gl_SecondaryFragColorEXT");
            }

            if (contextVersion == 1) continue;

            debug(`ESSL 3.00: ${label}`);

            {
                const none = `#version 300 es
                    void main() {}`;
                wtu.setupProgram(gl, [wtu.simpleVertexShaderESSL300, none]);
                wtu.drawUnitQuad(gl);
                wtu.glErrorShouldBe(gl, maskedOut ? gl.NO_ERROR : gl.INVALID_OPERATION,
                                    "no fragment outputs");
            }

            {
                const color0 = `#version 300 es
                    out mediump vec4 color0;
                    void main() {
                        color0 = vec4(1.0, 0.0, 0.0, 1.0);
                    }`;
                wtu.setupProgram(gl, [wtu.simpleVertexShaderESSL300, color0]);
                wtu.drawUnitQuad(gl);
                wtu.glErrorShouldBe(gl, (!enabled || maskedOut) ? gl.NO_ERROR : gl.INVALID_OPERATION,
                                    "only index 0 output");
            }

            {
                const color1 = `#version 300 es
                    #extension GL_EXT_blend_func_extended : enable
                    layout(location = 0, index = 1) out mediump vec4 color1;
                    void main() {
                        color1 = vec4(0.0, 1.0, 0.0, 1.0);
                    }`;
                wtu.setupProgram(gl, [wtu.simpleVertexShaderESSL300, color1]);
                wtu.drawUnitQuad(gl);
                wtu.glErrorShouldBe(gl, maskedOut ? gl.NO_ERROR : gl.INVALID_OPERATION,
                                    "only index 1 output");
            }
        }
    }
    gl.colorMask(true, true, true, true);
}

function runDrawBuffersLimitTests() {
    const dbi = gl.getExtension("OES_draw_buffers_indexed");
    if (!dbi) return;

    debug("");
    debug("Testing that dual-source blending limits the number of active draw buffers");

    const rb0 = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, rb0);
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA8, 1, 1);

    const rb1 = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, rb1);
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA8, 1, 1);

    gl.bindRenderbuffer(gl.RENDERBUFFER, null);

    const fbo = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, rb0);
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT1, gl.RENDERBUFFER, rb1);
    wtu.framebufferStatusShouldBe(gl, gl.FRAMEBUFFER, gl.FRAMEBUFFER_COMPLETE);

    const fs = `#version 300 es
        #extension GL_EXT_blend_func_extended : enable
        layout(location = 0, index = 0) out mediump vec4 color0;
        layout(location = 0, index = 1) out mediump vec4 color1;
        void main() {
            color0 = vec4(1.0, 0.0, 0.0, 1.0);
            color1 = vec4(0.0, 1.0, 0.0, 1.0);
        }`;
    wtu.setupProgram(gl, [wtu.simpleVertexShaderESSL300, fs]);

    wtu.setupUnitQuad(gl);

    // Enable both draw buffers
    gl.drawBuffers([gl.COLOR_ATTACHMENT0, gl.COLOR_ATTACHMENT1]);

    // Mask out draw buffer 1 to pass missing fragment outputs check
    dbi.colorMaskiOES(1, false, false, false, false);

    const extFuncs = [
        "SRC1_COLOR_WEBGL",
        "SRC1_ALPHA_WEBGL",
        "ONE_MINUS_SRC1_COLOR_WEBGL",
        "ONE_MINUS_SRC1_ALPHA_WEBGL"
    ];

    for (const func of extFuncs) {
        for (let slot = 0; slot < 4; slot++) {
            let param;
            switch (slot) {
                case 0:
                    param = "srcRGB";
                    gl.blendFuncSeparate(ext[func], gl.ONE, gl.ONE, gl.ONE);
                    break;
                case 1:
                    param = "dstRGB";
                    gl.blendFuncSeparate(gl.ONE, ext[func], gl.ONE, gl.ONE);
                    break;
                case 2:
                    param = "srcAlpha";
                    gl.blendFuncSeparate(gl.ONE, gl.ONE, ext[func], gl.ONE);
                    break;
                case 3:
                    param = "dstAlpha";
                    gl.blendFuncSeparate(gl.ONE, gl.ONE, gl.ONE, ext[func]);
                    break;
            }
            debug(`Testing ${func} with ${param}`);

            // Limit must be applied even with blending disabled
            gl.disable(gl.BLEND);
            wtu.drawUnitQuad(gl);
            wtu.glErrorShouldBe(gl, gl.INVALID_OPERATION, "blending disabled");

            gl.enable(gl.BLEND);
            wtu.drawUnitQuad(gl);
            wtu.glErrorShouldBe(gl, gl.INVALID_OPERATION, "blending enabled");

            // Limit is not applied when non-SRC1 funcs are used
            gl.blendFunc(gl.ONE, gl.ONE);
            wtu.drawUnitQuad(gl);
            wtu.glErrorShouldBe(gl, gl.NO_ERROR, "dual-source blending disabled");
        }
    }
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
}

function runBlendingTests() {
    debug("");
    debug("Testing rendering with two most common dual-source blending configurations");

    const fs = `#extension GL_EXT_blend_func_extended : enable
        uniform mediump vec4 u_src0;
        uniform mediump vec4 u_src1;
        void main() {
            gl_FragColor             = u_src0;
            gl_SecondaryFragColorEXT = u_src1;
        }`;
    const program = wtu.setupProgram(gl, [wtu.simpleVertexShader, fs]);
    const uSrc0 = gl.getUniformLocation(program, "u_src0");
    const uSrc1 = gl.getUniformLocation(program, "u_src1");

    gl.enable(gl.BLEND);
    wtu.setupUnitQuad(gl);
    gl.clearColor(1.0, 1.0, 1.0, 1.0);

    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.blendFunc(gl.ONE, ext.SRC1_COLOR_WEBGL);
    gl.uniform4f(uSrc0, 0.250, 0.375, 0.500, 0.625);
    gl.uniform4f(uSrc1, 0.125, 0.125, 0.125, 0.125);
    wtu.drawUnitQuad(gl);
    wtu.checkCanvas(gl, [96, 128, 159, 191], "Multiply destination by SRC1 and add SRC0", 2);

    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.blendFunc(ext.SRC1_COLOR_WEBGL, ext.ONE_MINUS_SRC1_COLOR_WEBGL);
    gl.uniform4f(uSrc0, 0.125, 0.125, 0.125, 0.125);
    gl.uniform4f(uSrc1, 0.500, 0.375, 0.250, 0.125);
    wtu.drawUnitQuad(gl);
    wtu.checkCanvas(gl, [143, 171, 199, 227], "Per-channel color interpolation using SRC1", 2);
}

function runTest() {
    if (!gl) {
        testFailed("context does not exist");
        return;
    }
    testPassed("context exists");

    runTestNoExtension();
    runShaderTests(false);

    ext = gl.getExtension("WEBGL_blend_func_extended");
    wtu.runExtensionSupportedTest(gl, "WEBGL_blend_func_extended", ext !== null);

    if (ext !== null) {
        runEnumTests();
        runQueryTests();
        runShaderTests(true);
        runMissingOutputsTests();
        runDrawBuffersLimitTests();
        runBlendingTests();
    } else {
        testPassed("No WEBGL_blend_func_extended support -- this is legal");
    }
}

runTest();

var successfullyParsed = true;
