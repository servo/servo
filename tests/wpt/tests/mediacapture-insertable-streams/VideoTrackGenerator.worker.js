// META: title=VideoTrackGenerator tests.

importScripts("/resources/testharness.js");

function make_audio_data(timestamp, channels, sampleRate, frames) {
  let data = new Float32Array(frames*channels);

  // This generates samples in a planar format.
  for (var channel = 0; channel < channels; channel++) {
    let hz = 100 + channel * 50; // sound frequency
    let base_index = channel * frames;
    for (var i = 0; i < frames; i++) {
      let t = (i / sampleRate) * hz * (Math.PI * 2);
      data[base_index + i] = Math.sin(t);
    }
  }

  return new AudioData({
    timestamp: timestamp,
    data: data,
    numberOfChannels: channels,
    numberOfFrames: frames,
    sampleRate: sampleRate,
    format: "f32-planar",
  });
}

const pixelColour = [50, 100, 150, 255];
const height = 240;
const width = 320;
function makeVideoFrame(timestamp) {
  const canvas = new OffscreenCanvas(width, height);

  const ctx = canvas.getContext('2d', {alpha: false});
  ctx.fillStyle = `rgba(${pixelColour.join()})`;
  ctx.fillRect(0, 0, width, height);

  return new VideoFrame(canvas, {timestamp, alpha: 'discard'});
}

promise_test(async t => {
  const videoFrame = makeVideoFrame(1);
  const originalWidth = videoFrame.displayWidth;
  const originalHeight = videoFrame.displayHeight;
  const originalTimestamp = videoFrame.timestamp;
  const generator = new VideoTrackGenerator();
  t.add_cleanup(() => generator.track.stop());

  // Use a MediaStreamTrackProcessor as a sink for |generator| to verify
  // that |processor| actually forwards the frames written to its writable
  // field.
  const processor = new MediaStreamTrackProcessor(generator);
  const reader = processor.readable.getReader();
  const readerPromise = new Promise(async resolve => {
    const result = await reader.read();
    t.add_cleanup(() => result.value.close());
    t.step_func(() => {
      assert_equals(result.value.displayWidth, originalWidth);
      assert_equals(result.value.displayHeight, originalHeight);
      assert_equals(result.value.timestamp, originalTimestamp);
    })();
    resolve();
  });

  generator.writable.getWriter().write(videoFrame);
  return readerPromise;
}, 'Tests that VideoTrackGenerator forwards frames to sink');

promise_test(async t => {
  const generator = new VideoTrackGenerator();
  t.add_cleanup(() => generator.track.stop());

  const writer = generator.writable.getWriter();
  const frame = makeVideoFrame(1);
  await writer.write(frame);

  assert_equals(generator.track.kind, "video");
  assert_equals(generator.track.readyState, "live");
}, "Tests that creating a VideoTrackGenerator works as expected");

promise_test(async t => {
  const generator = new VideoTrackGenerator();
  t.add_cleanup(() => generator.track.stop());

  const writer = generator.writable.getWriter();
  const frame = makeVideoFrame(1);
  await writer.write(frame);

  assert_throws_dom("InvalidStateError", () => frame.clone(), "VideoFrame wasn't destroyed on write.");
}, "Tests that VideoFrames are destroyed on write");

promise_test(async t => {
  const generator = new VideoTrackGenerator();
  t.add_cleanup(() => generator.track.stop());

  const writer = generator.writable.getWriter();

  if (!self.AudioData)
    return;

  const defaultInit = {
      timestamp: 1234,
      channels: 2,
      sampleRate: 8000,
      frames: 100,
  };
  const audioData = make_audio_data(defaultInit.timestamp, defaultInit.channels, defaultInit.sampleRate,
      defaultInit.frames);

  await promise_rejects_js(t, TypeError, writer.write("test"));
}, "Generator writer rejects on mismatched media input");

promise_test(async t => {
  const generator = new VideoTrackGenerator();
  t.add_cleanup(() => generator.track.stop());

  const writer = generator.writable.getWriter();
  await promise_rejects_js(t, TypeError, writer.write("potatoe"));
}, "Generator writer rejects on non media input");

promise_test(async t => {
  const generator = new VideoTrackGenerator();

  const writer = generator.writable.getWriter();
  const frame1 = makeVideoFrame(1);
  t.add_cleanup(() => frame1.close());
  await writer.write(frame1);
  assert_equals(frame1.codedWidth, 0);

  generator.track.stop();

  await writer.closed;

  const frame2 = makeVideoFrame(1);
  t.add_cleanup(() => frame2.close());
  await promise_rejects_js(t, TypeError, writer.write(frame2));

  assert_equals(frame2.codedWidth, 320);
}, "A writer rejects when generator's track is stopped");

promise_test(async t => {
  const generator = new VideoTrackGenerator();
  generator.muted = true;

  const writer = generator.writable.getWriter();
  const frame1 = makeVideoFrame(1);
  t.add_cleanup(() => frame1.close());
  await writer.write(frame1);
  assert_equals(frame1.codedWidth, 0);

  generator.track.stop();

  await writer.closed;

  const frame2 = makeVideoFrame(1);
  t.add_cleanup(() => frame2.close());
  await promise_rejects_js(t, TypeError, writer.write(frame2));

  assert_equals(frame2.codedWidth, 320);
}, "A muted writer rejects when generator's track is stopped");

promise_test(async t => {
  const generator = new VideoTrackGenerator();

  const writer = generator.writable.getWriter();
  const frame1 = makeVideoFrame(1);
  t.add_cleanup(() => frame1.close());
  await writer.write(frame1);
  assert_equals(frame1.codedWidth, 0);

  const clonedTrack = generator.track.clone();
  generator.track.stop();

  await new Promise(resolve => t.step_timeout(resolve, 100));

  const frame2 = makeVideoFrame(1);
  t.add_cleanup(() => frame2.close());
  await writer.write(frame2);
  assert_equals(frame2.codedWidth, 0);

  clonedTrack.stop();

  await writer.closed;

  const frame3 = makeVideoFrame(1);
  t.add_cleanup(() => frame3.close());
  await promise_rejects_js(t, TypeError, writer.write(frame3));

  assert_equals(frame3.codedWidth, 320);
}, "A writer rejects when generator's track and clones are stopped");

promise_test(async t => {
  const generator = new VideoTrackGenerator();
  t.add_cleanup(() => generator.track.stop());

  // Use a MediaStreamTrackProcessor as a sink for |generator| to verify
  // that |processor| actually forwards the frames written to its writable
  // field.
  const processor = new MediaStreamTrackProcessor(generator);
  const reader = processor.readable.getReader();
  const videoFrame = makeVideoFrame(1);

  const writer = generator.writable.getWriter();
  const videoFrame1 = makeVideoFrame(1);
  writer.write(videoFrame1);
  const result1 = await reader.read();
  t.add_cleanup(() => result1.value.close());
  assert_equals(result1.value.timestamp, 1);
  generator.muted = true;

  // This frame is expected to be discarded.
  const videoFrame2 = makeVideoFrame(2);
  writer.write(videoFrame2);
  generator.muted = false;

  const videoFrame3 = makeVideoFrame(3);
  writer.write(videoFrame3);
  const result3 = await reader.read();
  t.add_cleanup(() => result3.value.close());
  assert_equals(result3.value.timestamp, 3);

  // Set up a read ahead of time, then mute, enqueue and unmute.
  const promise5 = reader.read();
  generator.muted = true;
  writer.write(makeVideoFrame(4)); // Expected to be discarded.
  generator.muted = false;
  writer.write(makeVideoFrame(5));
  const result5 = await promise5;
  t.add_cleanup(() => result5.value.close());
  assert_equals(result5.value.timestamp, 5);
}, 'Tests that VideoTrackGenerator forwards frames only when unmuted');

done();
