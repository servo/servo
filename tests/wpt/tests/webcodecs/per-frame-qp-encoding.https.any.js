// META: global=window,dedicatedworker
// META: script=/webcodecs/video-encoder-utils.js
// META: variant=?av1
// META: variant=?vp9_p0
// META: variant=?vp9_p2

function get_config() {
  const config = {
    '?av1': {codec: 'av01.0.04M.08'},
    '?vp8': {codec: 'vp8'},
    '?vp9_p0': {codec: 'vp09.00.10.08'},
    '?vp9_p2': {codec: 'vp09.02.10.10'},
    '?h264': {codec: 'avc1.42001E', avc: {format: 'annexb'}}
  }[location.search];
  config.width = 320;
  config.height = 200;
  config.bitrate = 1000000;
  config.bitrateMode = 'quantizer';
  config.framerate = 30;
  return config;
}

function get_qp_range() {
  switch (location.search) {
    case '?av1':
      return {min: 1, max: 63};
    case '?vp9_p0':
      return {min: 1, max: 63};
    case '?vp9_p2':
      return {min: 1, max: 63};
    case '?h264':
      return {min: 1, max: 51};
  }
  return null;
}

function set_qp(options, value) {
  switch (location.search) {
    case '?av1':
      options.av1 = {quantizer: value};
      return;
    case '?vp9_p0':
      options.vp9 = {quantizer: value};
      return;
    case '?vp9_p2':
      options.vp9 = {quantizer: value};
      return;
    case '?h264':
      options.avc = {quantizer: value};
      return;
  }
}

async function per_frame_qp_test(t, encoder_config, qp_range, validate_result) {
  const w = encoder_config.width;
  const h = encoder_config.height;
  await checkEncoderSupport(t, encoder_config);

  const frames_to_encode = 12;
  let frames_decoded = 0;
  let frames_encoded = 0;
  let chunks = [];
  let corrupted_frames = [];

  const encoder_init = {
    output(chunk, metadata) {
      frames_encoded++;
      chunks.push(chunk);
    },
    error(e) {
      assert_unreached(e.message);
    }
  };

  let encoder = new VideoEncoder(encoder_init);
  encoder.configure(encoder_config);

  let qp = qp_range.min;
  for (let i = 0; i < frames_to_encode; i++) {
    let frame = createDottedFrame(w, h, i);
    let encode_options = {keyFrame: false};
    set_qp(encode_options, qp);
    encoder.encode(frame, encode_options);
    frame.close();
    qp += 3;
    if (qp > qp_range.max) {
      qp = qp_range.min
    }
  }
  await encoder.flush();

  let decoder = new VideoDecoder({
    output(frame) {
      frames_decoded++;
      // Check that we have intended number of dots and no more.
      // Completely black frame shouldn't pass the test.
      if (validate_result && !validateBlackDots(frame, frame.timestamp) ||
          validateBlackDots(frame, frame.timestamp + 1)) {
        corrupted_frames.push(frame.timestamp)
      }
      frame.close();
    },
    error(e) {
      assert_unreached(e.message);
    }
  });

  let decoder_config = {
    codec: encoder_config.codec,
    codedWidth: w,
    codedHeight: h,
  };
  decoder.configure(decoder_config);

  for (let chunk of chunks) {
    decoder.decode(chunk);
  }
  await decoder.flush();

  encoder.close();
  decoder.close();
  assert_equals(frames_encoded, frames_to_encode);
  assert_equals(chunks.length, frames_to_encode);
  assert_equals(frames_decoded, frames_to_encode);
  assert_equals(
      corrupted_frames.length, 0, `corrupted_frames: ${corrupted_frames}`);
}

promise_test(async t => {
  let config = get_config();
  let range = get_qp_range();
  return per_frame_qp_test(t, config, range, false);
}, 'Frame QP encoding, full range');

promise_test(async t => {
  let config = get_config();
  return per_frame_qp_test(t, config, {min: 1, max: 20}, true);
}, 'Frame QP encoding, good range with validation');
