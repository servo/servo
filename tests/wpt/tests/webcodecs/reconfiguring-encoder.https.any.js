// META: timeout=long
// META: global=window,dedicatedworker
// META: script=/webcodecs/video-encoder-utils.js
// META: variant=?av1
// META: variant=?vp8
// META: variant=?vp9_p0
// META: variant=?vp9_p2
// META: variant=?h264_avc
// META: variant=?h264_annexb

var ENCODER_CONFIG = null;
promise_setup(async () => {
  const config = {
    '?av1': {codec: 'av01.0.04M.08'},
    '?vp8': {codec: 'vp8'},
    '?vp9_p0': {codec: 'vp09.00.10.08'},
    '?vp9_p2': {codec: 'vp09.02.10.10'},
    '?h264_avc': {codec: 'avc1.42001F', avc: {format: 'avc'}},
    '?h264_annexb': {codec: 'avc1.42001F', avc: {format: 'annexb'}}
  }[location.search];
  config.hardwareAcceleration = 'prefer-software';
  config.bitrateMode = "constant";
  config.framerate = 30;
  ENCODER_CONFIG = config;
});

promise_test(async t => {
  let original_w = 800;
  let original_h = 600;
  let original_bitrate = 3_000_000;

  let new_w = 640;
  let new_h = 480;
  let new_bitrate = 2_000_000;

  let next_ts = 0
  let reconf_ts = 0;
  let frames_to_encode = 16;
  let before_reconf_frames = 0;
  let after_reconf_frames = 0;

  let process_video_chunk = function (chunk, metadata) {
    let config = metadata.decoderConfig;
    var data = new Uint8Array(chunk.data);
    assert_greater_than_equal(data.length, 0);
    let after_reconf = (reconf_ts != 0) && (chunk.timestamp >= reconf_ts);
    if (after_reconf) {
      after_reconf_frames++;
      if (config) {
        assert_equals(config.codedWidth, new_w);
        assert_equals(config.codedHeight, new_h);
      }
    } else {
      before_reconf_frames++;
      if (config) {
        assert_equals(config.codedWidth, original_w);
        assert_equals(config.codedHeight, original_h);
      }
    }
  };

  const init = {
    output: (chunk, md) => {
      try {
        process_video_chunk(chunk, md);
      } catch (e) {
        assert_unreached(e.message);
      }
    },
    error: (e) => {
      assert_unreached(e.message);
    },
  };
  const params = {
    ...ENCODER_CONFIG,
    width: original_w,
    height: original_h,
    bitrate: original_bitrate,
  };
  await checkEncoderSupport(t, params);

  let encoder = new VideoEncoder(init);
  encoder.configure(params);

  // Remove this flush after crbug.com/1275789 is fixed
  await encoder.flush();

  // Encode |frames_to_encode| frames with original settings
  for (let i = 0; i < frames_to_encode; i++) {
    var frame = createFrame(original_w, original_h, next_ts++);
    encoder.encode(frame, {});
    frame.close();
  }

  params.width = new_w;
  params.height = new_h;
  params.bitrate = new_bitrate;

  // Reconfigure encoder and run |frames_to_encode| frames with new settings
  encoder.configure(params);
  reconf_ts = next_ts;

  for (let i = 0; i < frames_to_encode; i++) {
    var frame = createFrame(new_w, new_h, next_ts++);
    encoder.encode(frame, {});
    frame.close();
  }

  await encoder.flush();

  // Configure back to original config
  params.width = original_w;
  params.height = original_h;
  params.bitrate = original_bitrate;
  encoder.configure(params);
  await encoder.flush();

  encoder.close();
  assert_equals(before_reconf_frames, frames_to_encode);
  assert_equals(after_reconf_frames, frames_to_encode);
}, "Reconfiguring encoder");
