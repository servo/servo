// META: global=window,dedicatedworker
// META: script=/webcodecs/videoFrame-utils.js
// META: script=/webcodecs/video-encoder-utils.js
// META: script=/webcodecs/videoFrame-encode-preserves-input-utils.js

// Encoding a VideoFrame must not rewrite the frame's own pixel data in place,
// regardless of the frame's pixel format. The frame is deliberately never
// rasterized before encoding.

const WIDTH = 64;
const HEIGHT = 64;

for (const format of ENCODE_FORMATS) {
  for (const codec of ENCODE_CODECS) {
    promise_test(async t => {
      const config = makeEncoderConfig(codec, WIDTH, HEIGHT);
      await checkEncoderSupport(t, config);

      const frame = new VideoFrame(makeFrameData(format, WIDTH, HEIGHT), {
        format,
        codedWidth: WIDTH,
        codedHeight: HEIGHT,
        timestamp: 0,
      });
      // clone() shares the frame's underlying image data.
      const clone = frame.clone();

      const frameBefore = await snapshot(frame);
      const cloneBefore = await snapshot(clone);

      await encode(frame, config,
                   t.unreached_func('unexpected encoder error'));

      assert_buffer_equals(await snapshot(frame), frameBefore);
      assert_buffer_equals(await snapshot(clone), cloneBefore);

      frame.close();
      clone.close();
    }, `Encoding a ${format} VideoFrame with ${codec.codec} preserves its ` +
       `input pixels`);
  }
}
