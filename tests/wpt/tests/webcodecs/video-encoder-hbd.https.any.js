// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js
// META: script=/webcodecs/video-encoder-utils.js
// META: variant=?av1
// META: variant=?vp8
// META: variant=?vp9
// META: variant=?vp9_p1
// META: variant=?vp9_p2
// META: variant=?vp9_p3
// META: variant=?h264

// Tests encoding high-bit-depth (16-bit) VideoFrame inputs.
// - Standard 8-bit codec profiles (AV1 Main 8-bit, VP8, VP9 Profile 0, VP9 Profile 1, H.264 Baseline)
//   perform 16-to-8 bit conversion prior to encoding.
// - High-bit-depth codec profiles (VP9 Profile 2, VP9 Profile 3) perform 16-bit high-bit-depth encoding.

const ENCODER_CONFIG = {
  '?av1': { codec: 'av01.0.04M.08', isHbd: false },
  '?vp8': { codec: 'vp8', isHbd: false },
  '?vp9': { codec: 'vp09.00.10.08', isHbd: false },
  '?vp9_p1': { codec: 'vp09.01.10.08.03', isHbd: false },
  '?vp9_p2': { codec: 'vp09.02.10.10', isHbd: true },
  '?vp9_p3': { codec: 'vp09.03.10.10.03', isHbd: true },
  '?h264': { codec: 'avc1.42001E', avc: { format: 'avc' }, isHbd: false },
}[location.search] || {};
ENCODER_CONFIG.width = 64;
ENCODER_CONFIG.height = 48;
ENCODER_CONFIG.bitrate = 1000000;
ENCODER_CONFIG.framerate = 30;

function createHbdFrame(format, width, height, ts) {
  let sub_w = width;
  let sub_h = height;
  if (format.includes('I420')) {
    sub_w = width / 2;
    sub_h = height / 2;
  } else if (format.includes('I422')) {
    sub_w = width / 2;
  }

  let luma_size = width * height;
  let chroma_size = sub_w * sub_h;
  let has_alpha = format.includes('I420A') || format.includes('I422A') || format.includes('I444A');
  let is_12bit = format.includes('P12');
  let shift = is_12bit ? 4 : 2;

  let total_samples = luma_size + 2 * chroma_size + (has_alpha ? luma_size : 0);
  let data = new Uint16Array(total_samples);

  let uOffset = luma_size;
  let vOffset = uOffset + chroma_size;
  let aOffset = vOffset + chroma_size;

  // Y=96, U=155, V=104
  data.fill(96 << shift, 0, uOffset);
  data.fill(155 << shift, uOffset, vOffset);
  data.fill(104 << shift, vOffset, has_alpha ? aOffset : total_samples);
  if (has_alpha) {
    let max_alpha = is_12bit ? 4095 : 1023;
    data.fill(max_alpha, aOffset);
  }

  return new VideoFrame(data, {
    format: format,
    timestamp: ts,
    codedWidth: width,
    codedHeight: height,
  });
}

const HBD_FORMATS = [
  'I420P10',
  'I422P10',
  'I444P10',
  'I420P12',
  'I422P12',
  'I444P12',
  'I420AP10',
  'I422AP10',
  'I444AP10',
];

HBD_FORMATS.forEach(format => {
  promise_test(async t => {
    let output_chunks = [];
    let decoder_config = null;
    let encoder_init = {
      output: (chunk, metadata) => {
        output_chunks.push(chunk);
        if (metadata && metadata.decoderConfig) {
          decoder_config = metadata.decoderConfig;
        }
      },
      error: (e) => {
        assert_unreached(e.message);
      }
    };

    await checkEncoderSupport(t, ENCODER_CONFIG);

    let encoder = new VideoEncoder(encoder_init);
    encoder.configure(ENCODER_CONFIG);

    let frame = createHbdFrame(format, ENCODER_CONFIG.width, ENCODER_CONFIG.height, 0);
    encoder.encode(frame);
    frame.close();

    await encoder.flush();
    encoder.close();

    assert_equals(output_chunks.length, 1, 'output chunk count');
    assert_greater_than(output_chunks[0].byteLength, 0, 'output chunk byte length');
    assert_not_equals(decoder_config, null, 'decoderConfig');

    let decoder_support = await VideoDecoder.isConfigSupported(decoder_config);
    assert_implements_optional(
        decoder_support.supported,
        'Unsupported decoder config: ' + JSON.stringify(decoder_config));

    let decoded_frame = null;
    let decoder_init = {
      output: (f) => {
        decoded_frame = f;
      },
      error: (e) => {
        assert_unreached(e.message);
      }
    };

    let decoder = new VideoDecoder(decoder_init);
    decoder.configure(decoder_config);
    decoder.decode(output_chunks[0]);
    await decoder.flush();
    decoder.close();

    assert_not_equals(decoded_frame, null, 'decoded frame');

    // Using copyTo({ format: 'RGBA' }) guarantees unified cross-browser readback
    // regardless of whether the decoder outputs software YUV or hardware GPU frames.
    let size = decoded_frame.allocationSize({ format: 'RGBA' });
    assert_greater_than(size, 0, 'allocationSize RGBA');
    let buffer = new ArrayBuffer(size);
    await decoded_frame.copyTo(buffer, { format: 'RGBA' });
    decoded_frame.close();

    let view = new Uint8Array(buffer);
    let tolerance = 25;
    assert_approx_equals(view[0], kSRGBPixel[0], tolerance, 'R pixel value');
    assert_approx_equals(view[1], kSRGBPixel[1], tolerance, 'G pixel value');
    assert_approx_equals(view[2], kSRGBPixel[2], tolerance, 'B pixel value');
    assert_approx_equals(view[3], 255, tolerance, 'A pixel value');
  }, ENCODER_CONFIG.isHbd
      ? `Test 16-bit high-bit-depth encoding of ${format}`
      : `Test 16-to-8 bit conversion encoding of ${format}`);
});
