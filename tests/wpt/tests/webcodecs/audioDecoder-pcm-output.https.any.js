// META: global=window,dedicatedworker
// META: variant=?pcm-u8
// META: variant=?pcm-s16
// META: variant=?pcm-s24
// META: variant=?pcm-s32
// META: variant=?pcm-f32

// Decodes a known signal encoded with each linear PCM codec of the WebCodecs
// PCM codec registration and checks every decoded sample, read back as
// f32-planar.

const SAMPLE_RATE = 48000;
const FRAMES = 480;
// One sine wave per channel, so that a channel swap is detected.
const FREQUENCIES = [440, 660];

function signal(channel, frame) {
  return 0.5 *
      Math.sin(2 * Math.PI * FREQUENCIES[channel] * frame / SAMPLE_RATE);
}

function writeInt24(view, offset, sample) {
  view.setUint8(offset, sample & 0xff);
  view.setUint8(offset + 1, (sample >> 8) & 0xff);
  view.setUint8(offset + 2, (sample >> 16) & 0xff);
}

// A K-bit integer sample is encoded as round(value * 2 ** (K - 1)), plus the
// bias 128 for u8. The spec gives the range of each sample type but not an
// exact conversion to f32, so a decoder may divide by 2 ** (K - 1) or by
// 2 ** (K - 1) - 1. For |value| <= 0.5, encoding rounds by up to 2 ** -K and
// the smaller divisor adds up to 1 / (2 ** K - 2), so the error stays below
// 1 / (2 ** (K - 1) - 1). Rounding the decoded and the expected sample to
// float32 adds less than 2 ** -23. pcm-f32 must decode unchanged.
function integerTolerance(bits) {
  return 1 / (2 ** (bits - 1) - 1) + 2 ** -23;
}

const FORMATS = {
  'pcm-u8': {
    bytesPerSample: 1,
    tolerance: integerTolerance(8),
    write: (view, offset, value) =>
        view.setUint8(offset, Math.round(value * 2 ** 7) + 128),
  },
  'pcm-s16': {
    bytesPerSample: 2,
    tolerance: integerTolerance(16),
    write: (view, offset, value) =>
        view.setInt16(offset, Math.round(value * 2 ** 15), true),
  },
  'pcm-s24': {
    bytesPerSample: 3,
    tolerance: integerTolerance(24),
    write: (view, offset, value) =>
        writeInt24(view, offset, Math.round(value * 2 ** 23)),
  },
  'pcm-s32': {
    bytesPerSample: 4,
    tolerance: integerTolerance(32),
    write: (view, offset, value) =>
        view.setInt32(offset, Math.round(value * 2 ** 31), true),
  },
  'pcm-f32': {
    bytesPerSample: 4,
    tolerance: 0,
    write: (view, offset, value) => view.setFloat32(offset, value, true),
  },
};

const CODEC = location.search.substring(1);
const FORMAT = FORMATS[CODEC];

function encode() {
  const channels = FREQUENCIES.length;
  const view = new DataView(
      new ArrayBuffer(FRAMES * channels * FORMAT.bytesPerSample));
  for (let frame = 0; frame < FRAMES; frame++) {
    for (let channel = 0; channel < channels; channel++) {
      FORMAT.write(
          view, (frame * channels + channel) * FORMAT.bytesPerSample,
          signal(channel, frame));
    }
  }
  return view.buffer;
}

promise_test(async t => {
  const config = {
    codec: CODEC,
    sampleRate: SAMPLE_RATE,
    numberOfChannels: FREQUENCIES.length,
  };
  const support = await AudioDecoder.isConfigSupported(config);
  assert_implements_optional(support.supported, CODEC + ' unsupported');

  const decoded = FREQUENCIES.map(() => []);
  const decoder = new AudioDecoder({
    output(data) {
      t.step(() => {
        assert_equals(
            data.numberOfChannels, FREQUENCIES.length, 'numberOfChannels');
        assert_equals(data.sampleRate, SAMPLE_RATE, 'sampleRate');
        for (let channel = 0; channel < FREQUENCIES.length; channel++) {
          const plane = new Float32Array(data.numberOfFrames);
          data.copyTo(plane, {planeIndex: channel, format: 'f32-planar'});
          decoded[channel].push(...plane);
        }
      });
      data.close();
    },
    error: t.unreached_func('unexpected error()'),
  });
  decoder.configure(config);
  decoder.decode(new EncodedAudioChunk({
    type: 'key',
    timestamp: 0,
    duration: 1000 * 1000 * FRAMES / SAMPLE_RATE,
    data: encode(),
  }));
  await decoder.flush();

  for (let channel = 0; channel < FREQUENCIES.length; channel++) {
    assert_equals(
        decoded[channel].length, FRAMES, `channel ${channel} frame count`);
    for (let frame = 0; frame < FRAMES; frame++) {
      const expected = Math.fround(signal(channel, frame));
      const label = `channel ${channel} frame ${frame}`;
      if (FORMAT.tolerance === 0) {
        assert_equals(decoded[channel][frame], expected, label);
      } else {
        assert_approx_equals(
            decoded[channel][frame], expected, FORMAT.tolerance, label);
      }
    }
  }
}, 'Test that decoded samples match the encoded input');
