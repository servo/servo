// META: script=resources/profile-utils.js

promise_test(async t => {
  assert_throws_js(TypeError, () => {
    new Profiler({ sampleInterval: 10 });
  });
}, 'max buffer size must be defined');

promise_test(async t => {
  const profiler = new Profiler({
    sampleInterval: 10,
    maxBufferSize: 2,
  });

  // Force 3 samples with a max buffer size of 2.
  for (let i = 0; i < 3; i++) {
    ProfileUtils.forceSample();
  }

  const trace = await profiler.stop();
  assert_equals(trace.samples.length, 2);
}, 'max buffer size is not exceeded');

promise_test(async t => {
  const largeBufferProfiler = new Profiler({ sampleInterval: 10,  maxBufferSize: Number.MAX_SAFE_INTEGER });
  const smallBufferProfiler = new Profiler({ sampleInterval: 10,  maxBufferSize: 1 });

  const watcher = new EventWatcher(t, smallBufferProfiler, ['samplebufferfull']);

  largeBufferProfiler.addEventListener('samplebufferfull', () => {
    assert_unreached('samplebufferfull invoked on wrong profiler');
    largeBufferProfiler.stop();
    smallBufferProfiler.stop();
  });

  smallBufferProfiler.addEventListener('samplebufferfull', () => {
    largeBufferProfiler.stop();
    smallBufferProfiler.stop();
  });

  // Force two samples to be taken, which should exceed
  // |smallBufferProfiler|'s buffer size.
  for (let i = 0; i < 2; i++) {
    ProfileUtils.forceSample();
  }

  // Ensure that |smallBufferProfiler|'s buffer size is exceeded.
  await watcher.wait_for('samplebufferfull');
}, 'ensure samplebufferfull is fired on full profiler');
