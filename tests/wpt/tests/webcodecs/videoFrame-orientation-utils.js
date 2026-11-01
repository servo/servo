// A 4x2 source image:
//   y y r r
//   b b g g
const kOrientationSourceWidth = 4;
const kOrientationSourceHeight = 2;
const kOrientationColors = {
  y: [255, 255, 0],
  r: [255, 0, 0],
  b: [0, 0, 255],
  g: [0, 255, 0],
};
const kOrientationSource = [
  ['y', 'y', 'r', 'r'],
  ['b', 'b', 'g', 'g'],
];

const kOrientationCombinations = [false, true].flatMap(flip => [0, 90, 180, 270].map(rotation => ({rotation, flip})));

function orientationDescription({rotation, flip}) {
  return `a ${rotation} degrees rotated${flip ? ' and flipped' : ''}`;
}

function makeOrientedFrame({rotation, flip = false}) {
  const data = new Uint8Array(kOrientationSourceWidth * kOrientationSourceHeight * 4);
  kOrientationSource.forEach((row, y) => row.forEach((name, x) => {
    data.set([...kOrientationColors[name], 255], 4 * (y * kOrientationSourceWidth + x));
  }));
  return new VideoFrame(data, {
    format: 'RGBA',
    codedWidth: kOrientationSourceWidth,
    codedHeight: kOrientationSourceHeight,
    timestamp: 0,
    rotation,
    flip,
  });
}

// Size of the source once rotated.
function orientedSize({rotation}) {
  return (rotation % 180) ? {width: kOrientationSourceHeight, height: kOrientationSourceWidth} :
                            {width: kOrientationSourceWidth, height: kOrientationSourceHeight};
}

// The name of the source color rendered at (x, y). The source is rotated clockwise, then flipped horizontally.
function expectedColorName({rotation, flip = false}, x, y) {
  if (flip)
    x = orientedSize({rotation}).width - 1 - x;
  const lastRow = kOrientationSourceHeight - 1;
  const lastColumn = kOrientationSourceWidth - 1;
  switch (rotation) {
    case 0: return kOrientationSource[y][x];
    case 90: return kOrientationSource[lastRow - x][y];
    case 180: return kOrientationSource[lastRow - y][lastColumn - x];
    case 270: return kOrientationSource[x][lastColumn - y];
  }
  assert_unreached('unexpected rotation');
}

// `pixels` is RGBA data of the rotated image. All pixels are checked.
function verifyOrientedPixels(pixels, orientation, description) {
  const {width, height} = orientedSize(orientation);
  assert_equals(pixels.length, width * height * 4, `${description}: pixels length`);
  for (let y = 0; y < height; ++y) {
    for (let x = 0; x < width; ++x) {
      const name = expectedColorName(orientation, x, y);
      const expected = kOrientationColors[name];
      const offset = 4 * (y * width + x);
      const actual = Array.from(pixels.subarray(offset, offset + 3));
      for (let i = 0; i < 3; ++i) {
        assert_approx_equals(actual[i], expected[i], 2,
            `${description}: pixel (${x}, ${y}) should be ${name} [${expected}] but is [${actual}]`);
      }
      assert_equals(pixels[offset + 3], 255, `${description}: pixel (${x}, ${y}) alpha`);
    }
  }
}

function verifyOrientedFrameProperties(frame, orientation) {
  const {width, height} = orientedSize(orientation);
  assert_equals(frame.rotation, orientation.rotation, 'rotation');
  assert_equals(frame.flip, !!orientation.flip, 'flip');
  assert_equals(frame.displayWidth, width, 'displayWidth');
  assert_equals(frame.displayHeight, height, 'displayHeight');
  assert_equals(frame.codedWidth, kOrientationSourceWidth, 'codedWidth');
  assert_equals(frame.codedHeight, kOrientationSourceHeight, 'codedHeight');
}

function runOrientationTests(name, test_function) {
  for (const orientation of kOrientationCombinations) {
    promise_test(async t => {
      const frame = makeOrientedFrame(orientation);
      t.add_cleanup(() => frame.close());
      verifyOrientedFrameProperties(frame, orientation);
      await test_function(frame, orientation, orientationDescription(orientation));
    }, `${name} of ${orientationDescription(orientation)} VideoFrame`);
  }
}

// Uploads `frame` to a texture with texImage2D() and reads it back as RGBA.
function readTexturePixels(frame, width, height) {
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  const gl = canvas.getContext('webgl', {alpha: true, antialias: false});
  assert_not_equals(gl, null, 'WebGL is available');

  const texture = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, false);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, frame);
  assert_equals(gl.getError(), gl.NO_ERROR, 'texImage2D');

  const framebuffer = gl.createFramebuffer();
  gl.bindFramebuffer(gl.FRAMEBUFFER, framebuffer);
  gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
  assert_equals(gl.checkFramebufferStatus(gl.FRAMEBUFFER), gl.FRAMEBUFFER_COMPLETE, 'framebuffer');

  const pixels = new Uint8Array(width * height * 4);
  gl.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
  assert_equals(gl.getError(), gl.NO_ERROR, 'readPixels');

  gl.deleteFramebuffer(framebuffer);
  gl.deleteTexture(texture);
  return pixels;
}
