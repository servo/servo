// META: title=MediaStreamTrackProcessor backpressure tests.

importScripts("/resources/testharness.js");

const height = 240;
const width = 320;

const inputCanvas = new OffscreenCanvas(width, height);
const inputCtx = inputCanvas.getContext('2d', {alpha: false});
inputCtx.fillStyle = 'black';
inputCtx.fillRect(0, 0, width, height);

const frameDuration = 40;

function makeUniformVideoFrame(timestamp) {
  return new VideoFrame(inputCanvas, {timestamp, alpha: 'discard'});
}

promise_test(async t => {
  const generator = new VideoTrackGenerator();
  t.add_cleanup(() => generator.track.stop());

  // Write frames for the duration of the test.
  const writer = generator.writable.getWriter();
  let timestamp = 0;
  const intervalId = setInterval(
    t.step_func(async () => {
      if (generator.readyState === 'live') {
        timestamp++;
        await writer.write(makeUniformVideoFrame(timestamp));
      }
    }),
    frameDuration);
  t.add_cleanup(() => clearInterval(intervalId));
  t.step_timeout(function() {
    clearInterval(intervalId);
    generator.track.stop();
  }, 2000);
  const processor = new MediaStreamTrackProcessor(generator);
  let ts = 1;
  await processor.readable.pipeTo(new WritableStream({
    async write(frame) {
      if (ts === 1) {
        assert_equals(frame.timestamp, ts, "Timestamp mismatch");
      } else {
        assert_greater_than_equal(frame.timestamp, ts, "Backpressure should have resulted in skipping at least 3 frames");
      }
      frame.close();
      ts+=3;
      // Wait the equivalent of 3 frames
      return new Promise((res) => t.step_timeout(res, 3*frameDuration));
    }
  }));
}, "Tests that backpressure forces MediaStreamTrackProcess to skip frames");

done();
