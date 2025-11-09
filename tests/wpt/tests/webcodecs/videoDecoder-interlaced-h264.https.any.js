// META: global=window,dedicatedworker
// META: script=videoDecoder-codec-specific-setup.js
// META: variant=?h264_interlaced_avc

promise_test(async t => {
  await checkImplements();

  const callbacks = {};
  const decoder = createVideoDecoder(t, callbacks);
  decoder.configure(CONFIG);
  decoder.decode(CHUNKS[0]);

  let outputs = 0;
  callbacks.output = frame => {
    outputs++;
    assert_equals(frame.timestamp, 0, 'timestamp');
    frame.close();
  };

  await decoder.flush();
  assert_equals(outputs, 1, 'outputs');
}, 'Test decoding h.264 interlaced content');
