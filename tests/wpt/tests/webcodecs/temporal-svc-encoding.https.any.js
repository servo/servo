// META: timeout=long
// META: global=window,dedicatedworker
// META: script=/webcodecs/video-encoder-utils.js
// META: variant=?av1
// META: variant=?vp8
// META: variant=?vp9
// META: variant=?h264

var ENCODER_CONFIG = null;
promise_setup(async () => {
  const config = {
    '?av1': {codec: 'av01.0.04M.08'},
    '?vp8': {codec: 'vp8'},
    '?vp9': {codec: 'vp09.00.10.08'},
    '?h264': {codec: 'avc1.42001E', avc: {format: 'annexb'}}
  }[location.search];
  config.hardwareAcceleration = 'prefer-software';
  config.width = 320;
  config.height = 200;
  config.bitrate = 1000000;
  config.bitrateMode = "constant";
  config.framerate = 30;
  ENCODER_CONFIG = config;
});

async function svc_test(t, layers, base_layer_decimator) {
  let encoder_config = { ...ENCODER_CONFIG };
  encoder_config.scalabilityMode = "L1T" + layers;
  const w = encoder_config.width;
  const h = encoder_config.height;
  await checkEncoderSupport(t, encoder_config);

  let frames_to_encode = 24;
  let frames_decoded = 0;
  let frames_encoded = 0;
  let chunks = [];
  let corrupted_frames = [];

  const encoder_init = {
    output(chunk, metadata) {
      frames_encoded++;

      // Filter out all frames, but base layer.
      assert_own_property(metadata, "svc");
      assert_own_property(metadata.svc, "temporalLayerId");
      assert_less_than(metadata.svc.temporalLayerId, layers);
      if (metadata.svc.temporalLayerId == 0)
        chunks.push(chunk);
    },
    error(e) {
      assert_unreached(e.message);
    }
  };

  let encoder = new VideoEncoder(encoder_init);
  encoder.configure(encoder_config);

  for (let i = 0; i < frames_to_encode; i++) {
    let frame = createDottedFrame(w, h, i);
    encoder.encode(frame, { keyFrame: false });
    frame.close();
  }
  await encoder.flush();

  let decoder = new VideoDecoder({
    output(frame) {
      frames_decoded++;
      // Check that we have intended number of dots and no more.
      // Completely black frame shouldn't pass the test.
      if(!validateBlackDots(frame, frame.timestamp) ||
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
    hardwareAcceleration: encoder_config.hardwareAcceleration,
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

  let base_layer_frames = frames_to_encode / base_layer_decimator;
  assert_equals(chunks.length, base_layer_frames);
  assert_equals(frames_decoded, base_layer_frames);
  assert_equals(corrupted_frames.length, 0,
    `corrupted_frames: ${corrupted_frames}`);
}

promise_test(async t => { return svc_test(t, 2, 2) }, "SVC L1T2");
promise_test(async t => { return svc_test(t, 3, 4) }, "SVC L1T3");
