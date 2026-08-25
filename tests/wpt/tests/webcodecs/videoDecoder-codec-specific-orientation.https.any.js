// META: global=window,dedicatedworker
// META: script=videoDecoder-codec-specific-setup.js
// META: variant=?av1
// META: variant=?vp8
// META: variant=?vp9
// META: variant=?h264_avc
// META: variant=?h264_annexb
// META: variant=?h265_hevc
// META: variant=?h265_annexb

promise_test(async t => {
  await checkImplements();
  const config = {
    ...CONFIG,
    rotation: 90,
    flip: true,
  };

  const support = await VideoDecoder.isConfigSupported(config);
  assert_true(support.supported, 'supported');
  assert_equals(support.config.rotation, config.rotation, 'rotation');
  assert_equals(support.config.flip, config.flip, 'flip');
}, 'Test that isConfigSupported() with orientation');

promise_test(async t => {
  await checkImplements();
  const callbacks = {};
  const decoder = createVideoDecoder(t, callbacks);
  let active_config = {
    ...CONFIG,
    rotation: 90,
    flip: true,
  };
  decoder.configure(active_config);
  decoder.decode(CHUNKS[0]);
  decoder.decode(CHUNKS[1]);

  let outputs = 0;
  callbacks.output = frame => {
    outputs++;
    assert_equals(frame.rotation, active_config.rotation, 'rotation');
    assert_equals(frame.flip, active_config.flip, 'flip');
    frame.close();
  };

  await decoder.flush();
  assert_equals(outputs, 2, 'outputs');

  // Reconfigure with a different orientation.
  active_config = {
    ...CONFIG,
    rotation: 180,
    flip: false,
  };
  decoder.configure(active_config);
  decoder.decode(CHUNKS[0]);
  decoder.decode(CHUNKS[1]);
  await decoder.flush();
  assert_equals(outputs, 4, 'outputs');
}, 'Decode frames with orientation metadata');
