// META: global=window,dedicatedworker
// META: script=/common/media.js
// META: script=/webcodecs/utils.js
// META: script=/webcodecs/video-encoder-utils.js

const defaultConfig = {
  codec: 'vp8',
  width: 640,
  height: 480
};

promise_test(async t => {
  let output_chunks = [];
  let codecInit = getDefaultCodecInit(t);
  let decoderConfig = null;
  codecInit.output = (chunk, metadata) => {
    assert_not_equals(metadata, null);
    if (metadata.decoderConfig)
      decoderConfig = metadata.decoderConfig;
    output_chunks.push(chunk);
  }

  let encoder = new VideoEncoder(codecInit);
  let config = defaultConfig;
  encoder.configure(config);

  let frame = createFrame(640, 480, 0, {rotation: 90, flip: true});
  encoder.encode(frame);
  frame.close();
  await encoder.flush();
  encoder.close();
  assert_equals(output_chunks.length, 1);
  assert_equals(decoderConfig.rotation, 90);
  assert_equals(decoderConfig.flip, true);
}, 'Encode video frame with orientation');

promise_test(async t => {
  let output_chunks = [];
  let codecInit = getDefaultCodecInit(t);
  let decoderConfig = null;
  codecInit.output = (chunk, metadata) => {
    assert_not_equals(metadata, null);
    if (metadata.decoderConfig)
      decoderConfig = metadata.decoderConfig;
    output_chunks.push(chunk);
  }

  let encoder = new VideoEncoder(codecInit);
  let config = defaultConfig;
  encoder.configure(config);

  let frame1 = createFrame(640, 480, 0, {rotation: 90, flip: true});
  let frame2 = createFrame(640, 480, 33333, {rotation: 90, flip: false});
  let frame3 = createFrame(640, 480, 66666, {rotation: 180, flip: true});
  let frame4 = createFrame(640, 480, 99999, {rotation: 90, flip: true});

  encoder.encode(frame1);
  assert_throws_dom('DataError', () => encoder.encode(frame2));
  assert_throws_dom('DataError', () => encoder.encode(frame3));
  encoder.encode(frame4);

  frame1.close();
  frame2.close();
  frame3.close();
  frame4.close();

  await encoder.flush();
  encoder.close();
  assert_equals(output_chunks.length, 2);
  assert_equals(decoderConfig.rotation, 90);
  assert_equals(decoderConfig.flip, true);
}, 'Encode video frames with different orientation has non-fatal failures');

promise_test(async t => {
  let output_chunks = [];
  let codecInit = getDefaultCodecInit(t);
  let decoderConfig = null;
  codecInit.output = (chunk, metadata) => {
    assert_not_equals(metadata, null);
    if (metadata.decoderConfig)
      decoderConfig = metadata.decoderConfig;
    output_chunks.push(chunk);
  }

  let encoder = new VideoEncoder(codecInit);
  let config = defaultConfig;
  encoder.configure(config);

  let frame = createFrame(640, 480, 0, {rotation: 90, flip: true});
  encoder.encode(frame);
  frame.close();
  await encoder.flush();
  assert_equals(output_chunks.length, 1);
  assert_equals(decoderConfig.rotation, 90);
  assert_equals(decoderConfig.flip, true);

  encoder.configure(config);
  frame = createFrame(640, 480, 0, {rotation: 270, flip: false});
  encoder.encode(frame);
  frame.close();
  await encoder.flush();
  assert_equals(output_chunks.length, 2);
  assert_equals(decoderConfig.rotation, 270);
  assert_equals(decoderConfig.flip, false);

  encoder.close();
}, 'Encode video frames with different orientations after reconfigure');
