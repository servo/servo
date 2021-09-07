// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js
// META: script=/webcodecs/webgl-test-utils.js

function testGLCanvas(gl, width, height, expectedPixel, assertCompares) {
  var colorData =
      new Uint8Array(gl.drawingBufferWidth * gl.drawingBufferHeight * 4);
  gl.readPixels(
      0, 0, gl.drawingBufferWidth, gl.drawingBufferHeight, gl.RGBA,
      gl.UNSIGNED_BYTE, colorData);
  assertCompares(gl.getError(), gl.NO_ERROR);

  const kMaxPixelToCheck = 128 * 96;
  let step = width * height / kMaxPixelToCheck;
  step = Math.round(step);
  step = (step < 1) ? 1 : step;
  for (let i = 0; i < 4 * width * height; i += (4 * step)) {
    assertCompares(colorData[i], expectedPixel[0]);
    assertCompares(colorData[i + 1], expectedPixel[1]);
    assertCompares(colorData[i + 2], expectedPixel[2]);
    assertCompares(colorData[i + 3], expectedPixel[3]);
  }
}

function testTexImage2DFromVideoFrame(
    width, height, useTexSubImage2D, expectedPixel) {
  let vfInit =
      {format: 'RGBA', timestamp: 0, codedWidth: width, codedHeight: height};
  let argbData = new Uint32Array(vfInit.codedWidth * vfInit.codedHeight);
  argbData.fill(0xFF966432);  // 'rgb(50, 100, 150)';
  let frame = new VideoFrame(argbData, vfInit);

  let gl_canvas = new OffscreenCanvas(width, height);
  let gl = gl_canvas.getContext('webgl');

  let program = WebGLTestUtils.setupTexturedQuad(gl);
  gl.clearColor(0, 0, 0, 1);
  gl.clearDepth(1);
  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
  gl.colorMask(1, 1, 1, 0);  // Disable any writes to the alpha channel.
  let textureLoc = gl.getUniformLocation(program, 'tex');

  let texture = gl.createTexture();

  // Bind the texture to texture unit 0.
  gl.bindTexture(gl.TEXTURE_2D, texture);

  // Set up texture parameters.
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

  // Set up pixel store parameters.
  gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, false);
  gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, false);

  // Upload the videoElement into the texture
  if (useTexSubImage2D) {
    // Initialize the texture to black first
    gl.texImage2D(
        gl.TEXTURE_2D, 0, gl.RGBA, width, height, 0, gl.RGBA, gl.UNSIGNED_BYTE,
        null);
    gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, frame);
  } else {
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, frame);
  }

  frame.close();

  assert_equals(gl.getError(), gl.NO_ERROR);

  // Point the uniform sampler to texture unit 0
  gl.uniform1i(textureLoc, 0);

  // Draw the triangles
  WebGLTestUtils.drawQuad(gl, [0, 0, 0, 255]);

  // Wait for drawing to complete.
  gl.finish();

  testGLCanvas(gl, width, height, expectedPixel, assert_equals);
}

function testTexImageWithClosedVideoFrame(useTexSubImage2D) {
  let width = 128;
  let height = 128;
  let vfInit =
      {format: 'RGBA', timestamp: 0, codedWidth: width, codedHeight: height};
  let argbData = new Uint32Array(vfInit.codedWidth * vfInit.codedHeight);
  argbData.fill(0xFF966432);  // 'rgb(50, 100, 150)';
  let frame = new VideoFrame(argbData, vfInit);

  let gl_canvas = new OffscreenCanvas(width, height);
  let gl = gl_canvas.getContext('webgl');

  frame.close();
  if (useTexSubImage2D) {
    gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, frame);
  } else {
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, frame);
  }

  assert_equals(gl.getError(), gl.INVALID_OPERATION);
}

test(_ => {
  testTexImage2DFromVideoFrame(48, 36, false, kSRGBPixel);
}, 'texImage2D with 48x36 srgb VideoFrame.');

test(_ => {
  testTexImage2DFromVideoFrame(48, 36, true, kSRGBPixel);
}, 'texSubImage2D with 48x36 srgb VideoFrame.');

test(_ => {
  testTexImage2DFromVideoFrame(480, 360, false, kSRGBPixel);
}, 'texImage2D with 480x360 srgb VideoFrame.');

test(_ => {
  testTexImage2DFromVideoFrame(480, 360, true, kSRGBPixel);
}, 'texSubImage2D with 480x360 srgb VideoFrame.');

test(_ => {
  testTexImageWithClosedVideoFrame(false);
}, 'texImage2D with a closed VideoFrame.');

test(_ => {
  testTexImageWithClosedVideoFrame(true);
}, 'texSubImage2D with a closed VideoFrame.');
