"use strict";
description("This test verifies the functionality of the EXT_texture_filter_anisotropic extension, if it is available.");

debug("");

let wtu = WebGLTestUtils;
let canvas = document.getElementById("canvas");
let gl = wtu.create3DContext(canvas, undefined, contextVersion);
let ext = null;
let sampler;

if (!gl) {
    testFailed("WebGL context does not exist");
} else {
    testPassed("WebGL context exists");

    // Run tests with extension disabled
    runHintTestDisabled();
    if (contextVersion >= 2) {
        runSamplerTestDisabled();
    }

    // Query the extension and store globally so shouldBe can access it
    ext = wtu.getExtensionWithKnownPrefixes(gl, "EXT_texture_filter_anisotropic");

    if (!ext) {
        testPassed("No EXT_texture_filter_anisotropic support -- this is legal");

        runSupportedTest(false);
    } else {
        testPassed("Successfully enabled EXT_texture_filter_anisotropic extension");

        runSupportedTest(true);
        runHintTestEnabled();
        if (contextVersion >= 2) {
            runSamplerTestEnabled();
        }
    }
}

function runSupportedTest(extensionEnabled) {
    if (wtu.getSupportedExtensionWithKnownPrefixes(gl, "EXT_texture_filter_anisotropic") !== undefined) {
        if (extensionEnabled) {
            testPassed("EXT_texture_filter_anisotropic listed as supported and getExtension succeeded");
        } else {
            testFailed("EXT_texture_filter_anisotropic listed as supported but getExtension failed");
        }
    } else {
        if (extensionEnabled) {
            testFailed("EXT_texture_filter_anisotropic not listed as supported but getExtension succeeded");
        } else {
            testPassed("EXT_texture_filter_anisotropic not listed as supported and getExtension failed -- this is legal");
        }
    }
}

function runHintTestDisabled() {
    debug("Testing MAX_TEXTURE_MAX_ANISOTROPY_EXT with extension disabled");

    const MAX_TEXTURE_MAX_ANISOTROPY_EXT = 0x84FF;
    shouldBeNull(`gl.getParameter(${MAX_TEXTURE_MAX_ANISOTROPY_EXT})`);
    wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, "MAX_TEXTURE_MAX_ANISOTROPY_EXT should not be queryable if extension is disabled");

    debug("Testing TEXTURE_MAX_ANISOTROPY_EXT with extension disabled");
    const TEXTURE_MAX_ANISOTROPY_EXT = 0x84FE;
    let texture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, texture);

    shouldBeNull(`gl.getTexParameter(gl.TEXTURE_2D, ${TEXTURE_MAX_ANISOTROPY_EXT})`);
    wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, "TEXTURE_MAX_ANISOTROPY_EXT should not be queryable if extension is disabled");

    gl.texParameterf(gl.TEXTURE_2D, TEXTURE_MAX_ANISOTROPY_EXT, 1);
    wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, "TEXTURE_MAX_ANISOTROPY_EXT should not be settable if extension is disabled");

    gl.texParameteri(gl.TEXTURE_2D, TEXTURE_MAX_ANISOTROPY_EXT, 1);
    wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, "TEXTURE_MAX_ANISOTROPY_EXT should not be settable if extension is disabled");

    gl.deleteTexture(texture);
}

function runHintTestEnabled() {
    debug("Testing MAX_TEXTURE_MAX_ANISOTROPY_EXT with extension enabled");

    shouldBe("ext.MAX_TEXTURE_MAX_ANISOTROPY_EXT", "0x84FF");

    let max_anisotropy = gl.getParameter(ext.MAX_TEXTURE_MAX_ANISOTROPY_EXT);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "MAX_TEXTURE_MAX_ANISOTROPY_EXT query should succeed if extension is enabled");

    if (max_anisotropy >= 2) {
        testPassed("Minimum value of MAX_TEXTURE_MAX_ANISOTROPY_EXT is 2.0");
    } else {
        testFailed("Minimum value of MAX_TEXTURE_MAX_ANISOTROPY_EXT is 2.0, returned values was: " + max_anisotropy);
    }

    // TODO make a texture and verify initial value == 1 and setting to less than 1 is invalid value

    debug("Testing TEXTURE_MAX_ANISOTROPY_EXT with extension disabled");
    shouldBe("ext.TEXTURE_MAX_ANISOTROPY_EXT", "0x84FE");

    let texture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, texture);

    let queried_value = gl.getTexParameter(gl.TEXTURE_2D, ext.TEXTURE_MAX_ANISOTROPY_EXT);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "TEXTURE_MAX_ANISOTROPY_EXT query should succeed if extension is enabled");

    if (queried_value == 1) {
        testPassed("Initial value of TEXTURE_MAX_ANISOTROPY_EXT is 1.0");
    } else {
        testFailed("Initial value of TEXTURE_MAX_ANISOTROPY_EXT should be 1.0, returned value was: " + queried_value);
    }

    gl.texParameterf(gl.TEXTURE_2D, ext.TEXTURE_MAX_ANISOTROPY_EXT, 0);
    wtu.glErrorShouldBe(gl, gl.INVALID_VALUE, "texParameterf TEXTURE_MAX_ANISOTROPY_EXT set to < 1 should be an invalid value");

    gl.texParameteri(gl.TEXTURE_2D, ext.TEXTURE_MAX_ANISOTROPY_EXT, 0);
    wtu.glErrorShouldBe(gl, gl.INVALID_VALUE, "texParameteri TEXTURE_MAX_ANISOTROPY_EXT set to < 1 should be an invalid value");

    gl.texParameterf(gl.TEXTURE_2D, ext.TEXTURE_MAX_ANISOTROPY_EXT, max_anisotropy);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "texParameterf TEXTURE_MAX_ANISOTROPY_EXT set to >= 2 should succeed");

    gl.texParameteri(gl.TEXTURE_2D, ext.TEXTURE_MAX_ANISOTROPY_EXT, max_anisotropy);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "texParameteri TEXTURE_MAX_ANISOTROPY_EXT set to >= 2 should succeed");

    queried_value = gl.getTexParameter(gl.TEXTURE_2D, ext.TEXTURE_MAX_ANISOTROPY_EXT);
    if (queried_value == max_anisotropy) {
        testPassed("Set value of TEXTURE_MAX_ANISOTROPY_EXT matches expecation");
    } else {
        testFailed("Set value of TEXTURE_MAX_ANISOTROPY_EXT should be: " + max_anisotropy + " , returned value was: " + queried_value);
    }

    gl.texParameterf(gl.TEXTURE_2D, ext.TEXTURE_MAX_ANISOTROPY_EXT, 1.5);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "texParameterf TEXTURE_MAX_ANISOTROPY_EXT set to 1.5 should succeed");

    queried_value = gl.getTexParameter(gl.TEXTURE_2D, ext.TEXTURE_MAX_ANISOTROPY_EXT);
    if (queried_value == 1.5) {
        testPassed("Set value of TEXTURE_MAX_ANISOTROPY_EXT matches expecation");
    } else {
        testFailed("Set value of TEXTURE_MAX_ANISOTROPY_EXT should be: " + 1.5 + " , returned value was: " + queried_value);
    }

    gl.deleteTexture(texture);
}

function runSamplerTestDisabled() {
    sampler = gl.createSampler();
    const TEXTURE_MAX_ANISOTROPY_EXT = 0x84FE;
    gl.samplerParameterf(sampler, TEXTURE_MAX_ANISOTROPY_EXT, 1.0);
    wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, "setting TEXTURE_MAX_ANISOTROPY_EXT on sampler without extension enabled should fail");
    shouldBeNull(`gl.getSamplerParameter(sampler, ${TEXTURE_MAX_ANISOTROPY_EXT})`);
    wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, "querying TEXTURE_MAX_ANISOTROPY_EXT on sampler without extension enabled should fail");
    gl.deleteSampler(sampler);
}

function runSamplerTestEnabled() {
    let sampler = gl.createSampler();
    gl.samplerParameterf(sampler, ext.TEXTURE_MAX_ANISOTROPY_EXT, 1.5);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "setting TEXTURE_MAX_ANISOTROPY_EXT on sampler should succeed");
    let queried_value = gl.getSamplerParameter(sampler, ext.TEXTURE_MAX_ANISOTROPY_EXT);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "querying TEXTURE_MAX_ANISOTROPY_EXT on sampler should succeed");
    if (queried_value == 1.5) {
        testPassed("Set value of TEXTURE_MAX_ANISOTROPY_EXT on sampler matches expecation");
    } else {
        testFailed("Set value of TEXTURE_MAX_ANISOTROPY_EXT on sampler should be: " + 1.5 + " , returned value was: " + queried_value);
    }
    gl.deleteSampler(sampler);
}

debug("");
var successfullyParsed = true;
