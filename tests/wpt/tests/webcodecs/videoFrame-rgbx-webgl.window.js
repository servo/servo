// META: script=/webcodecs/videoFrame-rgbx-utils.js

// Uploads `frame` to a texture with texImage2D() and reads it back as RGBA.
// `format` is the format of the texture, gl.RGBA or gl.RGB. The alpha of a
// gl.RGB texture reads back as opaque.
function readTexturePixels(frame, premultiplyAlpha, format = 'RGBA') {
  const canvas = document.createElement('canvas');
  canvas.width = kRgbxWidth;
  canvas.height = kRgbxHeight;
  const gl = canvas.getContext(
      'webgl', {alpha: true, premultipliedAlpha: false, antialias: false});
  assert_not_equals(gl, null, 'WebGL is available');

  const texture = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, premultiplyAlpha);
  gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, false);
  gl.texImage2D(
      gl.TEXTURE_2D, 0, gl[format], gl[format], gl.UNSIGNED_BYTE, frame);
  assert_equals(gl.getError(), gl.NO_ERROR, 'texImage2D');

  // Read the texture through a framebuffer to see its exact RGBA content.
  const framebuffer = gl.createFramebuffer();
  gl.bindFramebuffer(gl.FRAMEBUFFER, framebuffer);
  gl.framebufferTexture2D(
      gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
  assert_equals(
      gl.checkFramebufferStatus(gl.FRAMEBUFFER), gl.FRAMEBUFFER_COMPLETE,
      'framebuffer');

  const pixels = new Uint8Array(kRgbxWidth * kRgbxHeight * 4);
  gl.readPixels(
      0, 0, kRgbxWidth, kRgbxHeight, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
  assert_equals(gl.getError(), gl.NO_ERROR, 'readPixels');

  gl.deleteFramebuffer(framebuffer);
  gl.deleteTexture(texture);
  return pixels;
}

for (const premultiplyAlpha of [false, true]) {
  runRgbxTests(
      `texImage2D() (premultiplyAlpha=${premultiplyAlpha})`,
      (frame, description) => {
        verifyOpaqueRgbxPixels(
            readTexturePixels(frame, premultiplyAlpha), description);
      });
}

// With a gl.RGB texture the alpha of the frame is not uploaded at all: the RGB
// values must be the same as with gl.RGBA, whatever the padding byte is.
for (const premultiplyAlpha of [false, true]) {
  runRgbxTests(
      `texImage2D() as RGB (premultiplyAlpha=${premultiplyAlpha})`,
      (frame, description) => {
        verifyOpaqueRgbxPixels(
            readTexturePixels(frame, premultiplyAlpha, 'RGB'), description);
      });
}

// A cloned frame is uploaded the same way.
promise_test(async t => {
  const frame = makeRgbxFrame('RGBX', 0x00);
  t.add_cleanup(() => frame.close());
  const clone = frame.clone();
  t.add_cleanup(() => clone.close());
  verifyOpaqueRgbxPixels(readTexturePixels(clone, false), 'cloned frame');
}, 'texImage2D() of a cloned RGBX VideoFrame is opaque');
