self.onconnect = (e) => {
  const port = e.ports[0];
  port.onmessage = async (e) => {
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
      port.postMessage({url: url, success: result});
    } catch (err) {
      port.postMessage({url: url, success: false, error: err.name});
    }
  };
};
