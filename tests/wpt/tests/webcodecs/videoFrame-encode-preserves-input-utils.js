// Shared helpers for the encode input-preservation tests (clone-local and
// same-site cross-origin) across several formats and codecs. Uses only standard
// WebCodecs APIs (no privileged or gecko-specific APIs) so it can also run in
// the cross-origin child document.

const ENCODE_FORMATS = ['NV12', 'I420', 'RGBA'];

// The test image: a grid of solid blocks cycling through the primaries
// Red/Green/Blue. Every format encodes this SAME picture, so the frames differ
// only in pixel layout, not colour. BLOCK only needs to be even: that keeps
// each 4:2:0 chroma 2x2 group inside a single block (exact subsampled colour),
// and the grid tiles any even width/height (a partial edge block is fine) since
// blockColor() rounds the column count up. The primaries are fully saturated
// (never a grey/achromatic block where U == V == 128): distinct U and V are
// what make the NV12 interleaved-UV -> planar-UV reorder observable through
// copyTo.
const BLOCK = 16;
const PALETTE = [
  [255, 0, 0],  // red
  [0, 255, 0],  // green
  [0, 0, 255],  // blue
];

// BT.601 full-range (JFIF) RGB -> YUV, rounded and clamped to a byte. (Clamping
// matters: e.g. red's V and blue's U compute to 255.5.)
function rgb2yuv(r, g, b) {
  const clamp = v => Math.max(0, Math.min(255, Math.round(v)));
  return {
    y: clamp(0.299 * r + 0.587 * g + 0.114 * b),
    u: clamp(-0.168736 * r - 0.331264 * g + 0.5 * b + 128),
    v: clamp(0.5 * r - 0.418688 * g - 0.081312 * b + 128),
  };
}

// The palette colour of the block covering pixel (x, y). Math.ceil keeps the
// column count integral for any width (a non-multiple of BLOCK just yields a
// narrower final block column), so BLOCK need not divide the dimensions.
function blockColor(x, y, width) {
  const cols = Math.ceil(width / BLOCK);
  const index = (((y / BLOCK) | 0) * cols + ((x / BLOCK) | 0)) % PALETTE.length;
  return PALETTE[index];
}

// Builds the rotating-primaries image laid out for |format| (NV12, I420, RGBA).
// The colorSpace is intentionally left unset on the VideoFrame: the tests only
// byte-compare copyTo() output of a frame against itself, never decode it, so
// only the shared conversion's internal consistency matters.
function makeFrameData(format, width, height) {
  if (format !== 'NV12' && format !== 'I420' && format !== 'RGBA') {
    throw new Error(`unknown format: ${format}`);
  }
  // 4:2:0 (NV12/I420) subsamples chroma by 2, so both dimensions must be even.
  if (!Number.isInteger(width) || !Number.isInteger(height) || width <= 0 ||
      height <= 0 || width % 2 !== 0 || height % 2 !== 0) {
    throw new Error(
        `width and height must be positive even integers, got ${width}x${height}`);
  }
  const ySize = width * height;

  if (format === 'RGBA') {
    const data = new Uint8Array(ySize * 4);
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const [r, g, b] = blockColor(x, y, width);
        const o = (y * width + x) * 4;
        data[o] = r;
        data[o + 1] = g;
        data[o + 2] = b;
        data[o + 3] = 255;
      }
    }
    return data;
  }

  // NV12 and I420: full-resolution Y plane, then 4:2:0 chroma.
  const cw = width / 2;
  const ch = height / 2;
  const chromaSize = cw * ch;
  const data = new Uint8Array(ySize + 2 * chromaSize);
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const [r, g, b] = blockColor(x, y, width);
      data[y * width + x] = rgb2yuv(r, g, b).y;
    }
  }
  for (let cy = 0; cy < ch; cy++) {
    for (let cx = 0; cx < cw; cx++) {
      const [r, g, b] = blockColor(cx * 2, cy * 2, width);
      const {u, v} = rgb2yuv(r, g, b);
      const n = cy * cw + cx;
      if (format === 'NV12') {
        data[ySize + 2 * n] = u;      // interleaved: U, V, U, V, ...
        data[ySize + 2 * n + 1] = v;
      } else {
        data[ySize + n] = u;              // planar: U plane ...
        data[ySize + chromaSize + n] = v;  // ... then V plane
      }
    }
  }
  return data;
}

async function snapshot(frame) {
  const buf = new Uint8Array(frame.allocationSize());
  await frame.copyTo(buf);
  return buf;
}

// Codecs to exercise. Codec-specific config (e.g. H.264's `avc` block,
// which selects an avcC-format bitstream) flows through to configure().
const ENCODE_CODECS = [
  {codec: 'vp8'},
  {codec: 'avc1.42001E', avc: {format: 'avc'}},
];

function makeEncoderConfig(codec, width, height) {
  return {width, height, bitrate: 1000000, framerate: 30, ...codec};
}

// Encodes a single keyframe using |config|, a full VideoEncoderConfig passed
// straight to configure(). |onError| is the VideoEncoder error callback if
// provided.
async function encode(frame, config, onError) {
  let capturedError = null;
  const encoder = new VideoEncoder({
    output: () => {},
    error: onError || (e => {
             capturedError = e;
           }),
  });
  encoder.configure(config);
  encoder.encode(frame, {keyFrame: true});
  await encoder.flush();
  encoder.close();
  if (capturedError) {
    throw capturedError;
  }
}
