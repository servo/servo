self.onconnect = (e) => {
  const port = e.ports[0];
  port.onmessage = async (e) => {
    const url = e.data;
    try {
      const wt = new WebTransport(url);
      await wt.ready;
      wt.close();
      port.postMessage({url: url, success: true});
    } catch (err) {
      port.postMessage({url: url, success: false, error: err.name});
    }
  };
};
