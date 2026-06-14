self.addEventListener('message', async (e) => {
  const url = e.data;
  try {
    const ws = new WebSocket(url);
    const result = await new Promise(resolve => {
      ws.onopen = () => {
        ws.close();
        resolve(true);
      };
      ws.onerror = () => resolve(false);
    });
    e.source.postMessage({success: result});
  } catch (err) {
    e.source.postMessage({success: false, error: err.name});
  }
});
