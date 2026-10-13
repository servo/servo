// META: script=resources/profile-utils.js

promise_test(async () => {
  const start = performance.now();

  const profiler = new Profiler({
    sampleInterval: 10,
    maxBufferSize: Number.MAX_SAFE_INTEGER,
  });
  ProfileUtils.forceSample();
  const trace = await profiler.stop();

  const end = performance.now();

  assert_greater_than(trace.samples.length, 0);
  for (const sample of trace.samples) {
    assert_between_inclusive(sample.timestamp, start, end);
    const us = Math.round(sample.timestamp * 1000);
    const expectedResolution = self.crossOriginIsolated ? 5 : 100;
    assert_equals(
        us % expectedResolution, 0,
        `sample timestamp (${sample.timestamp} ms / ${
            us} us) should align with resolution of ${expectedResolution} us`);
  }
}, 'sample timestamps use the current high-resolution time');
