// META: script=/webcodecs/videoFrame-orientation-utils.js

// An 8x8 I420 frame made of four 4x4 blocks:
//   y r
//   b g
// The chroma planes are subsampled and upsampled with interpolation, which blends the colors
// next to the edge of a block. Only the pixels away from the edges are checked.
const kBlocks = [
  ['y', 'r'],
  ['b', 'g'],
];

// BT.709, limited range.
function rgbToYuv([r, g, b]) {
  const kr = 0.2126, kb = 0.0722, kg = 1 - kr - kb;
  const y = (kr * r + kg * g + kb * b) / 255;
  const u = (b / 255 - y) / (2 * (1 - kb));
  const v = (r / 255 - y) / (2 * (1 - kr));
  return [16 + 219 * y, 128 + 224 * u, 128 + 224 * v].map(Math.round);
}

const kSize = 8;
const kBlockSize = kSize / 2;

function makeI420Frame(rotation, flip) {
  const chromaSize = kSize / 2;
  const data = new Uint8Array(kSize * kSize + 2 * chromaSize * chromaSize);
  for (let y = 0; y < kSize; ++y) {
    for (let x = 0; x < kSize; ++x)
      data[y * kSize + x] = rgbToYuv(kOrientationColors[kBlocks[Math.floor(y / kBlockSize)][Math.floor(x / kBlockSize)]])[0];
  }
  const uOffset = kSize * kSize;
  const vOffset = uOffset + chromaSize * chromaSize;
  for (let y = 0; y < chromaSize; ++y) {
    for (let x = 0; x < chromaSize; ++x) {
      const [, u, v] = rgbToYuv(kOrientationColors[kBlocks[Math.floor(y / (kBlockSize / 2))][Math.floor(x / (kBlockSize / 2))]]);
      data[uOffset + y * chromaSize + x] = u;
      data[vOffset + y * chromaSize + x] = v;
    }
  }
  return new VideoFrame(data, {
    format: 'I420',
    codedWidth: kSize,
    codedHeight: kSize,
    timestamp: 0,
    rotation,
    flip,
  });
}

// The name of the color of the block rendered at the pixel (x, y), the blocks being rotated clockwise then flipped horizontally.
function expectedBlockName(rotation, flip, x, y) {
  if (flip)
    x = kSize - 1 - x;
  const blockX = Math.floor(x / kBlockSize);
  const blockY = Math.floor(y / kBlockSize);
  switch (rotation) {
    case 0: return kBlocks[blockY][blockX];
    case 90: return kBlocks[1 - blockX][blockY];
    case 180: return kBlocks[1 - blockY][1 - blockX];
    case 270: return kBlocks[blockX][1 - blockY];
  }
  assert_unreached('unexpected rotation');
}

for (const {rotation, flip} of kOrientationCombinations) {
  promise_test(async t => {
    const frame = makeI420Frame(rotation, flip);
    t.add_cleanup(() => frame.close());
    assert_equals(frame.rotation, rotation, 'rotation');
    assert_equals(frame.flip, flip, 'flip');
    assert_equals(frame.displayWidth, kSize, 'displayWidth');
    assert_equals(frame.displayHeight, kSize, 'displayHeight');

    const pixels = readTexturePixels(frame, kSize, kSize);
    // Skip the pixels next to the edges of the blocks.
    const isAwayFromEdge = coordinate => [1, 2].includes(coordinate % kBlockSize);
    for (let y = 0; y < kSize; ++y) {
      for (let x = 0; x < kSize; ++x) {
        if (!isAwayFromEdge(x) || !isAwayFromEdge(y))
          continue;
        const name = expectedBlockName(rotation, flip, x, y);
        const expected = kOrientationColors[name];
        const offset = 4 * (y * kSize + x);
        const actual = Array.from(pixels.subarray(offset, offset + 3));
        for (let i = 0; i < 3; ++i) {
          assert_approx_equals(actual[i], expected[i], 8,
              `pixel (${x}, ${y}) should be ${name} [${expected}] but is [${actual}]`);
        }
      }
    }
  }, `texImage2D() of ${orientationDescription({rotation, flip})} I420 VideoFrame`);
}
