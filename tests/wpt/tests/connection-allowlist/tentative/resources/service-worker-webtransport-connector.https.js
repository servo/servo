self.addEventListener('message', async (e) => {
  const url = e.data;
  try {
    const wt = new WebTransport(url);
    await wt.ready;
    wt.close();
    e.source.postMessage({success: true});
  } catch (err) {
    e.source.postMessage({success: false, error: err.name});
  }
});
