/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

'use strict';
description("Verifies that lines, both aliased and antialiased, have acceptable quality.");

let wtu = WebGLTestUtils;
let gl;

let aa_fbo;

function setupWebGL1Test(canvasId, useAntialiasing) {
  gl = wtu.create3DContext(canvasId, { antialias: useAntialiasing }, contextVersion);
  wtu.glErrorShouldBe(gl, gl.NO_ERROR, "Should be no errors during WebGL 1.0 setup");
}

function setupWebGL2Test(canvasId, useAntialiasing) {
  gl = wtu.create3DContext(canvasId, { antialias: false }, contextVersion);
  // In WebGL 2.0, we always allocate the back buffer without
  // antialiasing. The only question is whether we allocate a
  // framebuffer with a multisampled renderbuffer attachment.
  aa_fbo = null;
  if (useAntialiasing) {
    aa_fbo = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, aa_fbo);
    let rb = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, rb);
    let supported = gl.getInternalformatParameter(gl.RENDERBUFFER, gl.RGBA8, gl.SAMPLES);
    // Prefer 4, then 8, then max.
    let preferred = [4, 8];
    let allocated = false;
    for (let value of preferred) {
      if (supported.indexOf(value) >= 0) {
        gl.renderbufferStorageMultisample(gl.RENDERBUFFER, value, gl.RGBA8,
                                          gl.drawingBufferWidth, gl.drawingBufferHeight);
        allocated = true;
        break;
      }
    }
    if (!allocated) {
      gl.renderbufferStorageMultisample(gl.RENDERBUFFER, supported[supported.length - 1],
                                        gl.RGBA8, gl.drawingBufferWidth, gl.drawingBufferHeight);
    }
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, rb);
  }
  wtu.glErrorShouldBe(gl, gl.NO_ERROR, "Should be no errors during WebGL 2.0 setup");
}

function setupLines() {
  let prog = wtu.setupSimpleColorProgram(gl, 0);
  let loc = gl.getUniformLocation(prog, 'u_color');
  if (loc == null) {
    testFailed('Failed to fetch color uniform');
  }
  wtu.glErrorShouldBe(gl, gl.NO_ERROR, "After setup of line program");
  gl.uniform4f(loc, 1.0, 1.0, 1.0, 1.0);
  let buffer = gl.createBuffer();
  let scale = 0.5;
  gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
    -scale, -scale, 0.0, 1.0,
    -scale, scale, 0.0, 1.0,
    scale, scale, 0.0, 1.0,
    scale, -scale, 0.0, 1.0,
    -scale, -scale, 0.0, 1.0,
  ]), gl.STATIC_DRAW);
  wtu.glErrorShouldBe(gl, gl.NO_ERROR, "After setup of buffer");
  gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 0, 0);
  gl.enableVertexAttribArray(0);
  wtu.glErrorShouldBe(gl, gl.NO_ERROR, "After setup of attribute array");
}

function renderLines(contextVersion, useAntialiasing) {
  gl.clearColor(0.0, 0.0, 0.5, 1.0);
  gl.clear(gl.COLOR_BUFFER_BIT);
  gl.drawArrays(gl.LINE_STRIP, 0, 5);
  if (contextVersion == 2 && useAntialiasing) {
    // Blit aa_fbo into the real back buffer.
    gl.bindFramebuffer(gl.READ_FRAMEBUFFER, aa_fbo);
    gl.bindFramebuffer(gl.DRAW_FRAMEBUFFER, null);
    let w = gl.drawingBufferWidth;
    let h = gl.drawingBufferHeight;
    gl.blitFramebuffer(0, 0, w, h,
                       0, 0, w, h,
                       gl.COLOR_BUFFER_BIT, gl.NEAREST);
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
  }
}

function pixelAboveThreshold(arr, pixelIndex, threshold) {
  return (arr[4 * pixelIndex + 0] >= threshold &&
          arr[4 * pixelIndex + 1] >= threshold &&
          arr[4 * pixelIndex + 2] >= threshold &&
          arr[4 * pixelIndex + 3] >= threshold);
}

function checkLine(arr, threshold, direction) {
  // Count number of crossings from below threshold to above (or equal
  // to) threshold. Must be equal to 2.

  let numPixels = arr.length / 4;
  let numUpCrossings = 0;
  let numDownCrossings = 0;
  for (let index = 0; index < numPixels - 1; ++index) {
    let curPixel = pixelAboveThreshold(arr, index, threshold);
    let nextPixel = pixelAboveThreshold(arr, index + 1, threshold);
    if (!curPixel && nextPixel) {
      ++numUpCrossings;
    } else if (curPixel && !nextPixel) {
      ++numDownCrossings;
    }
  }
  if (numUpCrossings != numDownCrossings) {
    testFailed('Found differing numbers of up->down and down->up transitions');
  }
  if (numUpCrossings == 2) {
    testPassed('Found 2 lines, looking in the ' + direction + ' direction');
  } else {
    testFailed('Found ' + numUpCrossings + ' lines, looking in the ' +
               direction + ' direction, expected 2');
  }
}

function checkResults() {
  // Check the middle horizontal and middle vertical line of the canvas.
  let w = gl.drawingBufferWidth;
  let h = gl.drawingBufferHeight;
  let t = 100;
  let arr = new Uint8Array(4 * w);
  gl.readPixels(0, Math.floor(h / 2),
                w, 1, gl.RGBA, gl.UNSIGNED_BYTE, arr);
  checkLine(arr, t, 'horizontal');
  arr = new Uint8Array(4 * h);
  gl.readPixels(Math.floor(w / 2), 0,
                1, h, gl.RGBA, gl.UNSIGNED_BYTE, arr);
  checkLine(arr, t, 'vertical');
}

function runTest(contextVersion, canvasId, useAntialiasing) {
  switch (contextVersion) {
    case 1: {
      setupWebGL1Test(canvasId, useAntialiasing);
      break;
    }
    case 2: {
      setupWebGL2Test(canvasId, useAntialiasing);
    }
  }
  setupLines();
  renderLines(contextVersion, useAntialiasing);
  checkResults();
}

function runTests() {
  runTest(contextVersion, 'testbed', false);
  runTest(contextVersion, 'testbed2', true);
}

runTests();
let successfullyParsed = true;
