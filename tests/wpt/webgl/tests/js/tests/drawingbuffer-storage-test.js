/*
Copyright (c) 2023 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

"use strict";

let gl;
let oldViewport;
let width;
let height;
let format;
let hasDrawingBufferStorage;
let maxRenderbufferSize;

function runTest(contextVersion) {
  description();
  debug("");

  function initialize() {
    let canvas = document.createElement("canvas");
    gl = wtu.create3DContext(canvas, {antialias: false});
    if (!gl) {
      testFailed("context does not exist");
      return [0, 0];
    }

    hasDrawingBufferStorage = `drawingBufferStorage` in gl;
    if (!hasDrawingBufferStorage) {
      testPassed("drawingBufferStorage not present -- skipping test");
      return;
    }

    maxRenderbufferSize = gl.getParameter(gl.MAX_RENDERBUFFER_SIZE);
  }

  function testPixel(expected, actual, tol) {
    let str = 'approx equal: expected: ' + expected + ', actual: ' + actual + ', tolerance: ' + tol;
    for (let i = 0; i < 4; ++i) {
      if (Math.abs(expected[i] - actual[i]) > tol) {
        testFailed(str);
        return;
      }
    }
    testPassed(str);
  }

  function srgbToLinear(x) {
    if (x < 0.0)
      return 0.0;
    if (x < 0.04045)
      return x / 12.92;
    if (x < 1.0) {
      return Math.pow((x + 0.055)/1.044, 2.4);
    }
    return 1.0;
  }

  function testClearColor() {
    // Make a fresh canvas.
    let canvas = document.createElement("canvas");
    canvas.width = 16;
    canvas.height = 16;

    gl = wtu.create3DContext(canvas, {antialias: false});
    if (!gl) {
      testFailed("context does not exist");
      return;
    }
    testPassed("context exists");
    shouldBe('gl.drawingBufferFormat', 'gl.RGBA8');

    let testCase = function(f, size, clearColor, expectedPixel, tolerance) {
      format = f;
      width = size[0];
      height = size[1];

      gl.drawingBufferStorage(format, width, height);
      shouldBe('gl.getError()', 'gl.NO_ERROR');

      shouldBe('gl.drawingBufferFormat', 'format');
      shouldBe('gl.drawingBufferWidth', 'width');
      shouldBe('gl.drawingBufferHeight', 'height');

      gl.clearColor(clearColor[0], clearColor[1], clearColor[2], clearColor[3]);
      gl.clear(gl.COLOR_BUFFER_BIT);

      let buf;
      if (format == 0x881A /*RGBA16F*/) {
        buf = new Float32Array(4);
        gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.FLOAT, buf);
      } else {
        buf = new Uint8Array(4);
        gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, buf);
      }
      testPixel(expectedPixel, buf, tolerance);
    }

    debug('Testing RGBA8');
    testCase(gl.RGBA8, [16, 32],
             [16 / 255, 32 / 255, 64 / 255, 128 / 255],
             [16, 32, 64, 128],
             0);

    // WebGL 1 must use EXT_sRGB for SRGB8_ALPHA8.
    let srgb8_alpha8 = gl.SRGB8_ALPHA8;
    if (!srgb8_alpha8) {
      let ext = gl.getExtension('EXT_sRGB');
      if (ext) {
        srgb8_alpha8 = ext.SRGB8_ALPHA8_EXT;
      }
    }
    if (srgb8_alpha8) {
      debug('Testing SRGB8_ALPHA8');
      testCase(srgb8_alpha8, [16, 32],
               [srgbToLinear(64/255), srgbToLinear(16/255), srgbToLinear(32/255), 128 / 255],
               [64, 16, 32, 128],
               1);
    }

    if (gl.getExtension('EXT_color_buffer_float')) {
      // WebGL 1 must use EXT_color_buffer_half_float for RGBA16F.
      let rgba16f = gl.RGBA16F;
      if (!rgba16f) {
        let ext = gl.getExtension('EXT_color_buffer_half_float');
        if (ext) {
          rgba16f = ext.RGBA16F_EXT;
        }
      }

      debug('Testing RGBA16F');
      testCase(rgba16f, [18, 28],
               [0.25, 0.5, 0.75, 0.125],
               [0.25, 0.5, 0.75, 0.125],
               0.00001);
    } else {
      debug('Skipping RGBA16F');
    }

    shouldBe('gl.getError()', 'gl.NO_ERROR');
  }

  function testNoAlpha() {
    let canvas = document.createElement("canvas");
    canvas.width = 16;
    canvas.height = 16;
    gl = wtu.create3DContext(canvas, {alpha:false});
    if (!gl) {
      testFailed("context does not exist");
      return;
    }
    debug('Testing alpha:false');

    // Report RGB8 for the format.
    shouldBe('gl.drawingBufferFormat', 'gl.RGB8');

    // If WebGLContextAttributes.alpha is false, generate INVALID_OPERATION.
    gl.drawingBufferStorage(gl.RGBA8, 16, 16);
    shouldBe('gl.getError()', 'gl.INVALID_OPERATION');
  }

  function testMissingExtension() {
    let canvas = document.createElement("canvas");
    canvas.width = 16;
    canvas.height = 16;
    gl = wtu.create3DContext(canvas);
    if (!gl) {
      testFailed("context does not exist");
      return;
    }

    debug('Testing use of RGBA16F without enabling EXT_color_buffer_float');
    gl.drawingBufferStorage(gl.RGBA16F, 16, 16);
    shouldBe('gl.getError()', 'gl.INVALID_ENUM');
  }

  function testMaxSize() {
    let canvas = document.createElement("canvas");
    canvas.width = 16;
    canvas.height = 16;
    gl = wtu.create3DContext(canvas);
    if (!gl) {
      testFailed("context does not exist");
      return;
    }

    debug('Testing maximum size');
    gl.drawingBufferStorage(gl.RGBA8, maxRenderbufferSize, maxRenderbufferSize);
    shouldBe('gl.getError()', 'gl.NONE');
    shouldBe('gl.drawingBufferWidth', 'maxRenderbufferSize');
    shouldBe('gl.drawingBufferHeight', 'maxRenderbufferSize');

    debug('Testing over-maximum width and ehgith');
    gl.drawingBufferStorage(gl.RGBA8, maxRenderbufferSize+1, 16);
    shouldBe('gl.getError()', 'gl.INVALID_VALUE');
    gl.drawingBufferStorage(gl.RGBA8, 16, maxRenderbufferSize+1);
    shouldBe('gl.getError()', 'gl.INVALID_VALUE');
    shouldBe('gl.drawingBufferWidth', 'maxRenderbufferSize');
    shouldBe('gl.drawingBufferHeight', 'maxRenderbufferSize');
  }

  function testDrawToCanvas() {
    let canvasGL = document.createElement("canvas");
    canvasGL.width = 16;
    canvasGL.height = 16;
    gl = wtu.create3DContext(canvasGL);
    if (!gl) {
      testFailed("context does not exist");
      return;
    }

    let canvas2D = document.createElement("canvas");
    canvas2D.width = 16;
    canvas2D.height = 16;
    let ctx = canvas2D.getContext('2d');
    let imageData = new ImageData(16, 16);

    let testCase = function(f, clearColor, canvasColor, tolerance) {
      gl.drawingBufferStorage(f, 16, 16);
      gl.clearColor(clearColor[0], clearColor[1], clearColor[2], clearColor[3]);
      gl.clear(gl.COLOR_BUFFER_BIT);

      ctx.putImageData(imageData, 0, 0);
      ctx.drawImage(canvasGL, 0, 0);
      testPixel(canvasColor, ctx.getImageData(8, 8, 1, 1).data, tolerance);
    }

    debug('Drawing RGBA to canvas');
    testCase(gl.RGBA8, [16/255, 32/255, 64/255, 64/255], [64, 128, 255, 64], 0);

    // WebGL 1 must use EXT_sRGB for SRGB8_ALPHA8.
    let srgb8_alpha8 = gl.SRGB8_ALPHA8;
    if (!srgb8_alpha8) {
      let ext = gl.getExtension('EXT_sRGB');
      if (ext) {
        srgb8_alpha8 = ext.SRGB8_ALPHA8_EXT;
      }
    }
    if (srgb8_alpha8) {
      debug('Drawing opaque SRGB8_ALPHA8 to canvas');
      testCase(srgb8_alpha8,
               [srgbToLinear(64/255), srgbToLinear(32/255), srgbToLinear(16/255), 1.0],
               [64, 32, 16, 255],
               1);

      debug('Drawing transparent SRGB8_ALPHA8 to canvas');
      // We set the tolerance to 5 because of compounding error. The backbuffer
      // may be off by 1, and then un-premultiplying alpha of 64/55 will multiply
      // that error by 4. Then add one to be safe.
      testCase(srgb8_alpha8,
               [srgbToLinear(32/255), srgbToLinear(64/255), srgbToLinear(16/255), 64/255],
               [128, 255, 64, 64],
               5);
    }

    if (gl.getExtension('EXT_color_buffer_float')) {
      debug('Drawing transparent RGBA16F to canvas');
      testCase(gl.RGBA16F,
               [32/255, 64/255, 16/255, 64/255],
               [128, 255, 64, 64],
               1);
    }
  }

  let wtu = WebGLTestUtils;
  initialize();
  if (hasDrawingBufferStorage) {
    testClearColor();
    testNoAlpha();
    testMissingExtension();
    testMaxSize();
    testDrawToCanvas();
  }
}
