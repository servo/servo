onmessage = async (e) => {
  const url = e.data;
  try {
    const r = await fetch(url, { mode: 'cors', credentials: 'omit' });
    postMessage({ url: url, success: r.ok });
  } catch (err) {
    postMessage({ url: url, success: false, error: err.name });
  }
};
