// META: global=window,dedicatedworker

// The source is 16x16, with a different opaque color in each 8x8 quadrant.
const kSize = 16;
const kHalf = kSize / 2;
const kRed = [255, 0, 0, 255];
const kGreen = [0, 255, 0, 255];
const kBlue = [0, 0, 255, 255];
const kYellow = [255, 255, 0, 255];
const kTransparentBlack = [0, 0, 0, 0];

const kQuadrantCrop = {x: kHalf, y: kHalf, width: kHalf, height: kHalf};
const kOverhangingCrop = {x: kHalf, y: kHalf, width: kSize, height: kSize};
const kOutsideCrop = {x: kSize + kHalf, y: kHalf, width: kHalf, height: kHalf};

function sourceColor(x, y) {
  if (x < 0 || y < 0 || x >= kSize || y >= kSize) {
    return kTransparentBlack;
  }
  if (y < kHalf) {
    return x < kHalf ? kRed : kGreen;
  }
  return x < kHalf ? kBlue : kYellow;
}

function fillQuadrants(ctx) {
  for (const y of [0, kHalf]) {
    for (const x of [0, kHalf]) {
      const [r, g, b] = sourceColor(x, y);
      ctx.fillStyle = `rgb(${r}, ${g}, ${b})`;
      ctx.fillRect(x, y, kHalf, kHalf);
    }
  }
}

function makeOffscreenCanvas(contextOptions) {
  const canvas = new OffscreenCanvas(kSize, kSize);
  fillQuadrants(canvas.getContext('2d', contextOptions));
  return canvas;
}

function makeImageData() {
  return makeOffscreenCanvas().getContext('2d').getImageData(
      0, 0, kSize, kSize);
}

function cropImage(source, crop, options) {
  return createImageBitmap(
      source, crop.x, crop.y, crop.width, crop.height, options);
}

function verifyPixels(pixels, width, crop, description) {
  for (let y = 0; y < crop.height; y++) {
    for (let x = 0; x < crop.width; x++) {
      const i = (y * width + x) * 4;
      assert_array_equals(
          Array.from(pixels.subarray(i, i + 4)),
          sourceColor(crop.x + x, crop.y + y), `${description} at (${x}, ${y})`);
    }
  }
}

function verifyDrawImage(image, crop, description) {
  const ctx = new OffscreenCanvas(crop.width, crop.height).getContext('2d');
  ctx.drawImage(image, 0, 0);
  verifyPixels(
      ctx.getImageData(0, 0, crop.width, crop.height).data, crop.width, crop,
      description);
}

async function verifyCopyTo(frame, crop) {
  const options = {format: 'RGBA'};
  const buffer = new Uint8Array(frame.allocationSize(options));
  const layout = await frame.copyTo(buffer, options);
  assert_equals(layout.length, 1, 'number of planes');
  const {offset, stride} = layout[0];
  const rowBytes = crop.width * 4;
  const pixels = new Uint8Array(rowBytes * crop.height);
  for (let y = 0; y < crop.height; y++) {
    const start = offset + y * stride;
    pixels.set(buffer.subarray(start, start + rowBytes), y * rowBytes);
  }
  verifyPixels(pixels, crop.width, crop, 'copyTo({format: "RGBA"})');
}

async function verifyFrameFromCroppedBitmap(t, bitmap, crop) {
  t.add_cleanup(() => bitmap.close());
  assert_equals(bitmap.width, crop.width, 'bitmap.width');
  assert_equals(bitmap.height, crop.height, 'bitmap.height');
  verifyDrawImage(bitmap, crop, 'drawImage(bitmap)');

  const frame = new VideoFrame(bitmap, {timestamp: 0});
  t.add_cleanup(() => frame.close());
  assert_equals(frame.codedWidth, crop.width, 'codedWidth');
  assert_equals(frame.codedHeight, crop.height, 'codedHeight');
  assert_equals(frame.visibleRect.x, 0, 'visibleRect.x');
  assert_equals(frame.visibleRect.y, 0, 'visibleRect.y');
  assert_equals(frame.visibleRect.width, crop.width, 'visibleRect.width');
  assert_equals(frame.visibleRect.height, crop.height, 'visibleRect.height');
  assert_equals(frame.displayWidth, crop.width, 'displayWidth');
  assert_equals(frame.displayHeight, crop.height, 'displayHeight');
  verifyDrawImage(frame, crop, 'drawImage(frame)');
  await verifyCopyTo(frame, crop);
}

const kCases = [
  {
    name: 'OffscreenCanvas',
    make: crop => cropImage(makeOffscreenCanvas(), crop),
  },
  {
    name: 'OffscreenCanvas, premultiplyAlpha "premultiply"',
    make: crop => cropImage(
        makeOffscreenCanvas(), crop, {premultiplyAlpha: 'premultiply'}),
  },
  {
    name: 'OffscreenCanvas, premultiplyAlpha "none"',
    make: crop =>
        cropImage(makeOffscreenCanvas(), crop, {premultiplyAlpha: 'none'}),
  },
  {
    name: 'OffscreenCanvas, colorSpaceConversion "none"',
    make: crop => cropImage(
        makeOffscreenCanvas(), crop, {colorSpaceConversion: 'none'}),
  },
  {
    name: 'OffscreenCanvas, resizeWidth and resizeHeight',
    make: crop => cropImage(makeOffscreenCanvas(), crop, {
      resizeWidth: crop.width,
      resizeHeight: crop.height,
      resizeQuality: 'pixelated',
    }),
  },
  {
    name: 'opaque OffscreenCanvas, premultiplyAlpha "none"',
    make: crop => cropImage(
        makeOffscreenCanvas({alpha: false}), crop, {premultiplyAlpha: 'none'}),
  },
  {
    name: 'OffscreenCanvas, crop rect partly outside the source',
    crop: kOverhangingCrop,
    make: crop => cropImage(makeOffscreenCanvas(), crop),
  },
  {
    name: 'OffscreenCanvas, crop rect entirely outside the source',
    crop: kOutsideCrop,
    make: crop => cropImage(makeOffscreenCanvas(), crop),
  },
  {
    name:
        'OffscreenCanvas, premultiplyAlpha "none", crop rect entirely outside the source',
    crop: kOutsideCrop,
    make: crop =>
        cropImage(makeOffscreenCanvas(), crop, {premultiplyAlpha: 'none'}),
  },
  {
    name: 'ImageBitmap, crop rect entirely outside the source',
    crop: kOutsideCrop,
    make: async crop => {
      const source = makeOffscreenCanvas().transferToImageBitmap();
      const bitmap = await cropImage(source, crop);
      source.close();
      return bitmap;
    },
  },
  {
    name: 'ImageData',
    make: crop => cropImage(makeImageData(), crop),
  },
  {
    name: 'ImageBitmap',
    make: async crop => {
      const source = makeOffscreenCanvas().transferToImageBitmap();
      const bitmap = await cropImage(source, crop);
      source.close();
      return bitmap;
    },
  },
  {
    name: 'VideoFrame',
    make: async crop => {
      const source = new VideoFrame(makeOffscreenCanvas(), {timestamp: 0});
      const bitmap = await cropImage(source, crop);
      source.close();
      return bitmap;
    },
  },
  {
    name: 'OffscreenCanvas, after structuredClone',
    make: async crop => {
      const bitmap = await cropImage(makeOffscreenCanvas(), crop);
      const clone = structuredClone(bitmap);
      bitmap.close();
      return clone;
    },
  },
];

if (typeof document !== 'undefined') {
  kCases.push({
    name: 'HTMLCanvasElement',
    make: crop => {
      const canvas = document.createElement('canvas');
      canvas.width = kSize;
      canvas.height = kSize;
      fillQuadrants(canvas.getContext('2d'));
      return cropImage(canvas, crop);
    },
  });
}

for (const {name, crop = kQuadrantCrop, make} of kCases) {
  promise_test(async t => {
    await verifyFrameFromCroppedBitmap(t, await make(crop), crop);
  }, `VideoFrame from cropped ImageBitmap (${name})`);
}
