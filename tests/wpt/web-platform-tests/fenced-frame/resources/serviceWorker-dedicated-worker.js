self.addEventListener('message', async (e) => {
  if (e.data === 'fetch') {
    // Send a request to non-existing URL but handled by SW.
    const res = await fetch('./fenced_frame_dedicated_worker_test');
    const data = res.ok ? await res.text() : res.statusText;
    self.postMessage(data);
  }
});
