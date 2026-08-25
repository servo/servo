onconnect = (e) => {
  const port = e.ports[0];
  port.onmessage = async (e) => {
    const url = e.data;
    try {
      const r = await fetch(url, { mode: 'cors', credentials: 'omit' });
      port.postMessage({ url: url, success: r.ok });
    } catch (err) {
      port.postMessage({ url: url, success: false, error: err.name });
    }
  };
};
