"use strict";

function allocateTexture()
{
    var texture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "texture parameter setup should succeed");
    return texture;
}

function checkRenderingResults()
{
    wtu.checkCanvas(gl, [0, 255, 0, 255], "should be green");
}

function arrayToString(arr, size) {
    var mySize;
    if (!size)
        mySize = arr.length;
    else
        mySize = size;
    var out = "[";
    for (var ii = 0; ii < mySize; ++ii) {
    if (ii > 0) {
        out += ", ";
    }
    out += arr[ii];
    }
    return out + "]";
}

function runReadbackTest(testProgram, subtractor)
{
    // Verify floating point readback
    debug("Checking readback of floating-point values");
    var buf = new Float32Array(4);
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.FLOAT, buf);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "readPixels from floating-point framebuffer should succeed");
    var ok = true;
    var tolerance = 8.0; // TODO: factor this out from both this test and the subtractor shader above.
    for (var ii = 0; ii < buf.length; ++ii) {
        if (Math.abs(buf[ii] - subtractor[ii]) > tolerance) {
        ok = false;
        break;
        }
    }
    if (ok) {
        testPassed("readPixels of float-type data from floating-point framebuffer succeeded");
    } else {
        testFailed("readPixels of float-type data from floating-point framebuffer failed: expected "
                   + arrayToString(subtractor, 4) + ", got " + arrayToString(buf));
    }
}

function runFloatTextureRenderTargetTest(enabled, internalFormatString, format, type, testProgram, numberOfChannels, subtractor, texSubImageCover)
{
    let internalFormat = eval(internalFormatString);
    debug("");
    debug("testing floating-point " + internalFormatString + " texture render target" + (texSubImageCover > 0 ? " after calling texSubImage" : ""));

    var texture = allocateTexture();
    var width = 2;
    var height = 2;
    gl.texImage2D(gl.TEXTURE_2D, 0, internalFormat, width, height, 0, format, type, null);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "floating-point texture allocation should succeed");

    // Try to use this texture as a render target.
    var fbo = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
    gl.bindTexture(gl.TEXTURE_2D, null);

    var completeStatus = gl.checkFramebufferStatus(gl.FRAMEBUFFER);
    if (!enabled) {
        if (completeStatus == gl.FRAMEBUFFER_COMPLETE && !enabled)
            testFailed("floating-point " + internalFormatString + " render target should not be supported");
        else
            testPassed("floating-point " + internalFormatString + " render target should not be supported");
        return;
    }

    if (completeStatus != gl.FRAMEBUFFER_COMPLETE) {
        if (!wtu.isWebGL2(gl) && format == gl.RGB)
            testPassed("floating-point " + internalFormatString + " render target not supported; this is allowed.")
        else
            testFailed("floating-point " + internalFormatString + " render target not supported");
        return;
    }

    if (texSubImageCover > 0) {
        // Ensure that replacing the whole texture or a part of it with texSubImage2D doesn't affect renderability
        gl.bindTexture(gl.TEXTURE_2D, texture);
        var data = new Float32Array(width * height * numberOfChannels * texSubImageCover);
        gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, width, height * texSubImageCover, format, type, data);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "texSubImage2D should succeed if EXT_color_buffer_half_float is enabled");
        gl.bindTexture(gl.TEXTURE_2D, null);
        if (gl.checkFramebufferStatus(gl.FRAMEBUFFER) != gl.FRAMEBUFFER_COMPLETE) {
            testFailed("render target support changed after calling texSubImage2D");
            return;
        }
    }

    var renderProgram =
        wtu.setupProgram(gl,
                         [wtu.simpleVertexShader, `void main()
                                                    {
                                                        gl_FragColor = vec4(1000.0, 1000.0, 1000.0, 1000.0);
                                                    }`],
                         ['vPosition'],
                         [0]);
    wtu.clearAndDrawUnitQuad(gl);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "rendering to floating-point texture should succeed");

    // Now sample from the floating-point texture and verify we got the correct values.
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.useProgram(testProgram);
    gl.uniform1i(gl.getUniformLocation(testProgram, "tex"), 0);
    gl.uniform4fv(gl.getUniformLocation(testProgram, "subtractor"), subtractor);
    wtu.clearAndDrawUnitQuad(gl);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "rendering from floating-point texture should succeed");
    checkRenderingResults();

    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    runReadbackTest(testProgram, subtractor);
}

function runFloatRenderbufferRenderTargetTest(enabled, internalFormatString, testProgram, numberOfChannels, subtractor)
{
    var internalFormat = eval(internalFormatString);
    var samples = [0];
    if (enabled && wtu.isWebGL2(gl)) {
        samples = Array.prototype.slice.call(gl.getInternalformatParameter(gl.RENDERBUFFER, internalFormat, gl.SAMPLES));
        samples.push(0);
    }
    for (var ndx = 0; ndx < samples.length; ++ndx) {
        debug("");
        debug("testing floating-point " + internalFormatString + " renderbuffer render target with number of samples " + samples[ndx]);

        var colorbuffer = gl.createRenderbuffer();
        var width = 2;
        var height = 2;
        gl.bindRenderbuffer(gl.RENDERBUFFER, colorbuffer);
        if (samples[ndx] == 0)
            gl.renderbufferStorage(gl.RENDERBUFFER, internalFormat, width, height);
        else
            gl.renderbufferStorageMultisample(gl.RENDERBUFFER, samples[ndx], internalFormat, width, height);
        if (!enabled) {
            wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, "floating-point renderbuffer allocation should fail if EXT_color_buffer_half_float is not enabled or this is a 32 bit format");
            return;
        } else {
            wtu.glErrorShouldBe(gl, gl.NO_ERROR, "floating-point renderbuffer allocation should succeed if EXT_color_buffer_half_float is enabled");
        }

        // Try to use this renderbuffer as a render target.
        var fbo = gl.createFramebuffer();
        gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
        gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, colorbuffer);

        var completeStatus = gl.checkFramebufferStatus(gl.FRAMEBUFFER);
        if (completeStatus != gl.FRAMEBUFFER_COMPLETE) {
            testFailed("floating-point " + internalFormatString + " render target not supported");
            return;
        }
        var resolveColorRbo = null;
        var resolveFbo = null;
        if (samples[ndx] > 0) {
            resolveColorRbo = gl.createRenderbuffer();
            gl.bindRenderbuffer(gl.RENDERBUFFER, resolveColorRbo);
            gl.renderbufferStorage(gl.RENDERBUFFER, internalFormat, width, height);
            resolveFbo = gl.createFramebuffer();
            gl.bindFramebuffer(gl.FRAMEBUFFER, resolveFbo);
            gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, resolveColorRbo);
            completeStatus = gl.checkFramebufferStatus(gl.FRAMEBUFFER);
            if (completeStatus != gl.FRAMEBUFFER_COMPLETE) {
                testFailed("Failed to create resolve framebuffer");
                return;
            }
        }
        gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
        gl.clearColor(1000.0, 1000.0, 1000.0, 1000.0);
        gl.clear(gl.COLOR_BUFFER_BIT);

        if (samples[ndx] > 0) {
            gl.bindFramebuffer(gl.DRAW_FRAMEBUFFER, resolveFbo);
            gl.blitFramebuffer(0, 0, width, height, 0, 0, width, height, gl.COLOR_BUFFER_BIT, gl.NEAREST);
            gl.bindFramebuffer(gl.READ_FRAMEBUFFER, resolveFbo);
        }
        runReadbackTest(testProgram, subtractor);
    }
}

function runRGB16FNegativeTest()
{
    debug("");
    debug("testing RGB16F isn't color renderable");

    var texture = allocateTexture();
    var width = 2;
    var height = 2;
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB16F, width, height, 0, gl.RGB, gl.FLOAT, null);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "RGB16F texture allocation should succeed");

    // Try to use this texture as a render target.
    var fbo = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
    gl.bindTexture(gl.TEXTURE_2D, null);

    var completeStatus = gl.checkFramebufferStatus(gl.FRAMEBUFFER);
    if (completeStatus == gl.FRAMEBUFFER_COMPLETE)
        testFailed("RGB16F render target should not be supported with or without enabling EXT_color_buffer_half_float");
    else
        testPassed("RGB16F render target should not be supported with or without enabling EXT_color_buffer_half_float");
    gl.deleteTexture(texture);

    var colorbuffer = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, colorbuffer);
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGB16F, width, height);
    wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, "RGB16F renderbuffer allocation should fail with or without enabling EXT_color_buffer_half_float");
    gl.bindRenderbuffer(gl.RENDERBUFFER, null);
    gl.deleteRenderbuffer(colorbuffer);

    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    gl.deleteFramebuffer(fbo);
}

function runUniqueObjectTest()
{
    debug("");
    debug("Testing that getExtension() returns the same object each time");
    gl.getExtension("EXT_color_buffer_half_float").myProperty = 2;
    webglHarnessCollectGarbage();
    shouldBe('gl.getExtension("EXT_color_buffer_half_float").myProperty', '2');
}

function runInternalFormatQueryTest()
{
    debug("");
    debug("testing the internal format query");

    var maxSamples = gl.getParameter(gl.MAX_SAMPLES);
    const formats = [gl.RGBA16F, gl.R16F, gl.RG16F];
    var firstMultiOnlyFormat = 4;
    for (var fmt = 0; fmt < formats.length; ++fmt) {
        var samples = gl.getInternalformatParameter(gl.RENDERBUFFER, formats[fmt], gl.SAMPLES);
        if (fmt >= firstMultiOnlyFormat && (samples.length == 0 || samples[0] < maxSamples)) {
            testFailed("the maximum value in SAMPLES should be at least " + maxSamples);
            return;
        }

        var prevSampleCount = 0;
        var sampleCount;
        for (var ndx = 0; ndx < samples.length; ++ndx, prevSampleCount = sampleCount) {
            sampleCount = samples[ndx];
            // sample count must be > 0
            if (sampleCount <= 0) {
                testFailed("Expected sample count to be at least one; got " + sampleCount);
                return;
            }

            // samples must be ordered descending
            if (ndx > 0 && sampleCount >= prevSampleCount) {
                testFailed("Expected sample count to be ordered in descending order; got " + prevSampleCount + " at index " + (ndx - 1) + ", and " + sampleCount + " at index " + ndx);
                return;
            }
        }
    }
    testPassed("Internal format query succeeded");
}

function runCopyTexImageTest(enabled)
{
    var width = 16;
    var height = 16;
    var level = 0;
    var cases = [
        { internalformat: "RGBA16F", format: "RGBA", destFormat: "R16F", valid: true, renderable: true, },
        { internalformat: "RGBA16F", format: "RGBA", destFormat: "RG16F", valid: true, renderable: true, },
        { internalformat: "RGBA16F", format: "RGBA", destFormat: "RGB16F", valid: true, renderable: true, },
        { internalformat: "RGBA16F", format: "RGBA", destFormat: "RGBA16F", valid: true, renderable: true, },
        { internalformat: "RGB16F", format: "RGB", destFormat: "R16F", valid: true, renderable: false, },
        { internalformat: "RGB16F", format: "RGB", destFormat: "RG16F", valid: true, renderable: false, },
        { internalformat: "RGB16F", format: "RGB", destFormat: "RGB16F", valid: true, renderable: false, },
        { internalformat: "RGB16F", format: "RGB", destFormat: "RGBA16F", valid: false, renderable: false, },
        { internalformat: "RG16F", format: "RG", destFormat: "R16F", valid: true, renderable: true, },
        { internalformat: "RG16F", format: "RG", destFormat: "RG16F", valid: true, renderable: true, },
        { internalformat: "RG16F", format: "RG", destFormat: "RGB16F", valid: false, renderable: true, },
        { internalformat: "RG16F", format: "RG", destFormat: "RGBA16F", valid: false, renderable: true, },
        { internalformat: "R16F", format: "RED", destFormat: "R16F", valid: true, renderable: true, },
        { internalformat: "R16F", format: "RED", destFormat: "RG16F", valid: false, renderable: true, },
        { internalformat: "R16F", format: "RED", destFormat: "RGB16F", valid: false, renderable: true, },
        { internalformat: "R16F", format: "RED", destFormat: "RGBA16F", valid: false, renderable: true, },
    ];
    if (!wtu.isWebGL2(gl)) {
        cases = [
            { valid:  true, renderable: true, format: "RGBA",  destFormat: "LUMINANCE", },
            { valid:  true, renderable: true, format: "RGBA",  destFormat: "ALPHA", },
            { valid:  true, renderable: true, format: "RGBA",  destFormat: "LUMINANCE_ALPHA", },
            { valid:  true, renderable: true, format: "RGBA",  destFormat: "RGB", },
            { valid:  true, renderable: true, format: "RGBA",  destFormat: "RGBA", },
            { valid:  true, renderable: true, format: "RGB",   destFormat: "LUMINANCE", },
            { valid: false, renderable: true, format: "RGB",   destFormat: "ALPHA", },
            { valid: false, renderable: true, format: "RGB",   destFormat: "LUMINANCE_ALPHA", },
            { valid:  true, renderable: true, format: "RGB",   destFormat: "RGB", },
            { valid: false, renderable: true, format: "RGB",   destFormat: "RGBA", },
            { valid:  true, renderable: false, format: "ALPHA", destFormat: "ALPHA", },
            { valid:  true, renderable: false, format: "LUMINANCE", destFormat: "LUMINANCE", },
            { valid:  true, renderable: false, format: "LUMINANCE_ALPHA", destFormat: "LUMINANCE_ALPHA", },
        ];
    }
    cases.forEach(function(testcase) {
        debug("");
        debug(`Testing CopyTexImage2D for format: ${testcase.format}, internalformat: ${testcase.internalformat}, destformat: ${testcase.destFormat}`);

        var format = gl[testcase.format];
        var internalformat = wtu.isWebGL2(gl) ? gl[testcase.internalformat] : format;
        var type = wtu.isWebGL2(gl) ? gl.HALF_FLOAT : 0x8D61 /* HALF_FLOAT_OES */;
        var destFormat = gl[testcase.destFormat];
        var texSrc = gl.createTexture();
        gl.bindTexture(gl.TEXTURE_2D, texSrc);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        var data = new Uint16Array(width * height * 4);
        gl.texImage2D(gl.TEXTURE_2D, level, internalformat, width, height, 0, format, type, data);
        var fbo = gl.createFramebuffer();
        gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
        gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texSrc, level);
        var texDest = gl.createTexture();
        gl.bindTexture(gl.TEXTURE_2D, texDest);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "Setup framebuffer with texture should succeed.");
        if (enabled && testcase.renderable) {
            if (!wtu.isWebGL2(gl) && format == gl.RGB && gl.checkFramebufferStatus(gl.FRAMEBUFFER) == gl.FRAMEBUFFER_INCOMPLETE_ATTACHMENT) {
                testPassed("RGB framebuffer attachment not supported. This is allowed.")
            } else {
                shouldBe("gl.checkFramebufferStatus(gl.FRAMEBUFFER)", "gl.FRAMEBUFFER_COMPLETE");
                gl.copyTexImage2D(gl.TEXTURE_2D, level, destFormat, 0, 0, width, height, 0);
                wtu.glErrorShouldBe(gl, testcase.valid ? gl.NO_ERROR : [gl.INVALID_ENUM, gl.INVALID_OPERATION], "CopyTexImage2D");
            }
        } else {
            shouldBe("gl.checkFramebufferStatus(gl.FRAMEBUFFER)", "gl.FRAMEBUFFER_INCOMPLETE_ATTACHMENT");
            gl.copyTexImage2D(gl.TEXTURE_2D, level, destFormat, 0, 0, width, height, 0);
            wtu.glErrorShouldBe(gl, [gl.INVALID_ENUM, gl.INVALID_FRAMEBUFFER_OPERATION], "CopyTexImage2D should fail.");
        }

        gl.deleteTexture(texDest);
        gl.deleteTexture(texSrc);
        gl.deleteFramebuffer(fbo);
    });
}

description("This test verifies the functionality of the EXT_color_buffer_half_float extension, if it is available.");

debug("");

var wtu = WebGLTestUtils;
var canvas = document.getElementById("canvas");
var gl = wtu.create3DContext(canvas);

if (!wtu.isWebGL2(gl)) {
    // These are exposed on the extension, but we need them before the extension has been requested so we can
    // make sure they don't work.
    gl.R16F = 0x822D;
    gl.RG16F = 0x822F;
    gl.RGB16F = 0x881B;
    gl.RGBA16F = 0x881A;
}

if (!gl) {
  testFailed("WebGL context does not exist");
} else {
  testPassed("WebGL context exists");

  var texturedShaders = [
      wtu.simpleTextureVertexShader,
           `precision mediump float;
            uniform sampler2D tex;
            uniform vec4 subtractor;
            varying vec2 texCoord;
            void main()
            {
                vec4 color = texture2D(tex, texCoord);
                if (abs(color.r - subtractor.r) +
                    abs(color.g - subtractor.g) +
                    abs(color.b - subtractor.b) +
                    abs(color.a - subtractor.a) < 16.0) {
                    gl_FragColor = vec4(0.0, 1.0, 0.0, 1.0);
                } else {
                    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
                }
            }`,
  ];
  var testProgram =
      wtu.setupProgram(gl,
                       texturedShaders,
                       ['vPosition', 'texCoord0'],
                       [0, 1]);
  var quadParameters = wtu.setupUnitQuad(gl, 0, 1);

  if (wtu.isWebGL2(gl)) {
    // Ensure these formats can't be used for rendering if the extension is disabled
    runFloatTextureRenderTargetTest(false, "gl.R16F", gl.RED, gl.FLOAT);
    runFloatTextureRenderTargetTest(false, "gl.RG16F", gl.RG, gl.FLOAT);
    runFloatTextureRenderTargetTest(false, "gl.RGBA16F", gl.RGBA, gl.FLOAT);
  }

  runFloatRenderbufferRenderTargetTest(false, "gl.R16F");
  runFloatRenderbufferRenderTargetTest(false, "gl.RG16F");
  runFloatRenderbufferRenderTargetTest(false, "gl.RGBA16F");
  runFloatRenderbufferRenderTargetTest(false, "gl.R32F");
  runFloatRenderbufferRenderTargetTest(false, "gl.RG32F");
  runFloatRenderbufferRenderTargetTest(false, "gl.RGBA32F");
  runFloatRenderbufferRenderTargetTest(false, "gl.R11F_G11F_B10F");

  if (wtu.isWebGL2(gl)) {
      runCopyTexImageTest(false);
      // Ensure RGB16F can't be used for rendering.
      runRGB16FNegativeTest();
  }

  if (!wtu.isWebGL2(gl)) {
      debug("");
      debug("Testing that component type framebuffer attachment queries are rejected with the extension disabled");
      const fbo = gl.createFramebuffer();
      gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);

      const rbo = gl.createRenderbuffer();
      gl.bindRenderbuffer(gl.RENDERBUFFER, rbo);
      gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0,gl.RENDERBUFFER, rbo);
      gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGB565, 8, 8);
      wtu.glErrorShouldBe(gl, gl.NO_ERROR, "Setup renderbuffer should succeed.");
      shouldBeNull('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, 0x8211 /* FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE */)');
      wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, "Query must fail.");
      gl.deleteRenderbuffer(rbo);

      const tex = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, tex);
      gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 8, 8, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
      wtu.glErrorShouldBe(gl, gl.NO_ERROR, "Setup texture should succeed.");
      shouldBeNull('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, 0x8211 /* FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE */)');
      wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, "Query must fail.");
      gl.deleteTexture(tex);

      gl.deleteFramebuffer(fbo);
  }

  let oesTextureHalfFloat = null;
  if (!wtu.isWebGL2(gl)) {
    // oesTextureHalfFloat implicitly enables EXT_color_buffer_half_float if supported
    oesTextureHalfFloat = gl.getExtension("OES_texture_half_float");
    if (oesTextureHalfFloat && gl.getSupportedExtensions().includes("EXT_color_buffer_half_float")) {
        runFloatTextureRenderTargetTest(true, "gl.RGBA", gl.RGBA, oesTextureHalfFloat.HALF_FLOAT_OES, testProgram, 4, [1000, 1000, 1000, 1000], 0);
        runFloatTextureRenderTargetTest(true, "gl.RGB", gl.RGB, oesTextureHalfFloat.HALF_FLOAT_OES, testProgram, 3, [1000, 1000, 1000, 1], 0);
    }
  }

  var ext = null;
  if (!(ext = gl.getExtension("EXT_color_buffer_half_float"))) {
      testPassed("No EXT_color_buffer_half_float support -- this is legal");
  } else {
      testPassed("Successfully enabled EXT_color_buffer_half_float extension");

      shouldBe("ext.RGB16F_EXT", "gl.RGB16F");
      shouldBe("ext.RGBA16F_EXT", "gl.RGBA16F");
      shouldBe("ext.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE_EXT", "0x8211");
      shouldBe("ext.UNSIGNED_NORMALIZED_EXT", "0x8C17");

      if (wtu.isWebGL2(gl)) {
          runInternalFormatQueryTest();
          runFloatTextureRenderTargetTest(true, "gl.R16F", gl.RED, gl.FLOAT, testProgram, 1, [1000, 1, 1, 1], 0);
          runFloatTextureRenderTargetTest(true, "gl.RG16F", gl.RG, gl.FLOAT, testProgram, 2, [1000, 1000, 1, 1], 0);
          runFloatTextureRenderTargetTest(true, "gl.RGBA16F", gl.RGBA, gl.FLOAT, testProgram, 4, [1000, 1000, 1000, 1000], 0);
          runFloatRenderbufferRenderTargetTest(true, "gl.R16F", testProgram, 1, [1000, 1, 1, 1]);
          runFloatRenderbufferRenderTargetTest(true, "gl.RG16F", testProgram, 2, [1000, 1000, 1, 1]);
          runFloatRenderbufferRenderTargetTest(true, "gl.RGBA16F", testProgram, 4, [1000, 1000, 1000, 1000]);
      }
      if (!wtu.isWebGL2(gl)) {
          shouldBeNonNull(oesTextureHalfFloat); // Required by spec
          runFloatTextureRenderTargetTest(true, "gl.RGBA", gl.RGBA, oesTextureHalfFloat.HALF_FLOAT_OES, testProgram, 4, [1000, 1000, 1000, 1000], 0);
          runFloatTextureRenderTargetTest(true, "gl.RGB", gl.RGB, oesTextureHalfFloat.HALF_FLOAT_OES, testProgram, 3, [1000, 1000, 1000, 1], 0);
          runFloatTextureRenderTargetTest(false, "gl.LUMINANCE_ALPHA", gl.LUMINANCE_ALPHA, oesTextureHalfFloat.HALF_FLOAT_OES, testProgram, 2, [1000, 1000, 1000, 1000], 0);
          runFloatTextureRenderTargetTest(false, "gl.LUMINANCE", gl.LUMINANCE, oesTextureHalfFloat.HALF_FLOAT_OES, testProgram, 1, [1000, 1, 1, 1], 0);
      }

      if (wtu.isWebGL2(gl))
          runRGB16FNegativeTest(); // Ensure EXT_color_buffer_half_float does not enable RGB16F as color renderable.

      runCopyTexImageTest(true);

      runUniqueObjectTest();

      {
          debug("");
          debug("Testing that component type framebuffer attachment queries are accepted with the extension enabled");
          const fbo = gl.createFramebuffer();
          gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);

          const rbo = gl.createRenderbuffer();
          gl.bindRenderbuffer(gl.RENDERBUFFER, rbo);
          gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0,gl.RENDERBUFFER, rbo);
          gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGB565, 8, 8);
          shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, ext.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE_EXT)', 'ext.UNSIGNED_NORMALIZED_EXT');
          gl.renderbufferStorage(gl.RENDERBUFFER, ext.RGBA16F_EXT, 8, 8);
          shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, ext.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE_EXT)', 'gl.FLOAT');
          wtu.glErrorShouldBe(gl, gl.NO_ERROR, "No errors after valid renderbuffer attachment queries.");

          gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_STENCIL_ATTACHMENT,gl.RENDERBUFFER, rbo);
          gl.renderbufferStorage(gl.RENDERBUFFER, gl.DEPTH_STENCIL, 8, 8);
          wtu.glErrorShouldBe(gl, gl.NO_ERROR, "No errors after depth-stencil renderbuffer setup.");
          shouldBeNull('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.DEPTH_STENCIL_ATTACHMENT, ext.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE_EXT)');
          wtu.glErrorShouldBe(gl, gl.INVALID_OPERATION, "Component type query is not allowed for combined depth-stencil attachments.");
          gl.deleteRenderbuffer(rbo);

          const tex = gl.createTexture();
          gl.bindTexture(gl.TEXTURE_2D, tex);
          gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
          gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 8, 8, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
          shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, ext.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE_EXT)', 'ext.UNSIGNED_NORMALIZED_EXT');
          const tex_ext = gl.getExtension("OES_texture_half_float");
          if (wtu.isWebGL2(gl) || tex_ext) {
              if (wtu.isWebGL2(gl))
                  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA16F, 8, 8, 0, gl.RGBA, gl.HALF_FLOAT, null);
              else
                  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 8, 8, 0, gl.RGBA, tex_ext.HALF_FLOAT_OES, null);
              shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, ext.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE_EXT)', 'gl.FLOAT');
          }
          wtu.glErrorShouldBe(gl, gl.NO_ERROR, "No errors after valid texture attachment queries.");
          gl.deleteTexture(tex);

          gl.deleteFramebuffer(fbo);
      }
  }
}

debug("");
var successfullyParsed = true;
