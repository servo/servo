// META: global=window,dedicatedworker
// META: script=/common/media.js
// META: script=/webcodecs/utils.js
// META: script=/webcodecs/video-encoder-utils.js
// META: variant=?vp8
// META: variant=?h264_avc

const VP8_CONFIG = {
  codec: 'vp8',
  width: 640,
  height: 480,
  displayWidth: 800,
  displayHeight: 600,
};

const H264_AVC_CONFIG = {
  codec: 'avc1.42001e', // Baseline
  width: 640,
  height: 480,
  displayWidth: 800,
  displayHeight: 600,
  avc: {format: 'avc'},
};

let CONFIG = null;
promise_setup(async () => {
  CONFIG = {
    '?vp8': VP8_CONFIG,
    '?h264_avc': H264_AVC_CONFIG,
  }[location.search];
});

promise_test(async t => {
  let codecInit = getDefaultCodecInit(t);

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
  encoder.configure(CONFIG);

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

promise_test(async t => {
  let frame1 = createFrame(640, 480, 0);
  let frame2 = createFrame(640, 480, 33333);
  t.add_cleanup(() => {
    frame1.close();
    frame2.close();
  });

  const callbacks = {};
  const encoder = createVideoEncoder(t, callbacks);

  let flushInCallbackDone;
  let outputs = 0;
  let firstOutput = new Promise(resolve => {
    callbacks.output = (chunk, metadata) => {
      encoder.reset();

      callbacks.output = (chunk, metadata) => {
        outputs++;
      };

      encoder.configure(CONFIG);
      encoder.encode(frame2);
      flushInCallbackDone = encoder.flush();

      resolve();
    };
  });

  encoder.configure(CONFIG);
  encoder.encode(frame1);
  const flushDone = encoder.flush();

  // Wait for the first output, then reset.
  await firstOutput;

  // Flush should have been synchronously rejected.
  await promise_rejects_dom(t, 'AbortError', flushDone);

  // Wait for the second flush and check the output count.
  await flushInCallbackDone;

  assert_equals(outputs, 1, 'outputs');
}, 'Test new flush after reset in a flush callback');
