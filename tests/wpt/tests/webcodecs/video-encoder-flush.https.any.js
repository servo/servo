// META: global=window,dedicatedworker
// META: script=/common/media.js
// META: script=/webcodecs/utils.js
// META: script=/webcodecs/video-encoder-utils.js

promise_test(async t => {
  let codecInit = getDefaultCodecInit(t);
  let encoderConfig = {
    codec: 'vp8',
    width: 640,
    height: 480,
    displayWidth: 800,
    displayHeight: 600,
  };

  let outputs = 0;
  let firstOutput = new Promise(resolve => {
    codecInit.output = (chunk, metadata) => {
      outputs++;
      assert_equals(outputs, 1, 'outputs');
      encoder.reset();
      resolve();
    };
  });

  let encoder = new VideoEncoder(codecInit);
  encoder.configure(encoderConfig);

  let frame1 = createFrame(640, 480, 0);
  let frame2 = createFrame(640, 480, 33333);
  t.add_cleanup(() => {
    frame1.close();
    frame2.close();
  });

  encoder.encode(frame1);
  encoder.encode(frame2);
  const flushDone = encoder.flush();

  // Wait for the first output, then reset.
  await firstOutput;

  // Flush should have been synchronously rejected.
  await promise_rejects_dom(t, 'AbortError', flushDone);

  assert_equals(outputs, 1, 'outputs');
}, 'Test reset during flush');
