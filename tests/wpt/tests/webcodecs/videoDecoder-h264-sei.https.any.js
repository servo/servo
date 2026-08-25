// META: global=window,dedicatedworker
// META: script=videoDecoder-codec-specific-setup.js
// META: variant=?h264_sei_avc
// META: variant=?h264_sei_annexb

promise_test(async t => {
  await checkImplements();
  const callbacks = {};
  const decoder = createVideoDecoder(t, callbacks);
  decoder.configure(CONFIG);

  // Frame 0 is IDR, frame 5 is SEI recovery point.
  decoder.decode(CHUNKS[5]);

  // First decode the IDR frame to
  let outputs = 0;
  callbacks.output = frame => {
    outputs++;
    assert_equals(frame.timestamp, CHUNKS[5].timestamp, 'timestamp');
    assert_equals(frame.duration, CHUNKS[5].duration, 'duration');
    frame.close();
  };

  await decoder.flush();
  assert_equals(outputs, 1, 'outputs');
}, 'H.264 SEI recovery point frames are treated as keyframes.');
