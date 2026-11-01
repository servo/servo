importScripts("/resources/testharness.js");

function makeVideoFrame(timestamp) {
  const canvas = new OffscreenCanvas(100, 100);
  const ctx = canvas.getContext('2d');
  return new VideoFrame(canvas, {timestamp});
}

promise_test(async t => {
  const generator = new VideoTrackGenerator();
  const processor = new MediaStreamTrackProcessor({track: generator.track});

  const reader = processor.readable.getReader();
  const writer = generator.writable.getWriter();

  await new Promise(resolve => t.step_timeout(resolve, 0));
  writer.write(makeVideoFrame(0));
  generator.track.stop();

  await reader.read().finally(() => { });
}, "Stopping a track just after writing");

done();
