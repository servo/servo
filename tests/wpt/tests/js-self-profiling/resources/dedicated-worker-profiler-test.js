// Dedicated worker test for js-self-profiling with document policy
try {
  new Profiler({ sampleInterval: 10, maxBufferSize: Number.MAX_SAFE_INTEGER });
  self.postMessage({ success: true, message: 'Profiler created' });
} catch (e) {
  self.postMessage({ success: false, error: e.name, message: e.message });
}
