let received = new Map();
self.onmessage = (e) => {
  if (e.data.hasOwnProperty('id')) {
    e.source.postMessage(
      received.get(e.data.id) ? 'RECEIVED' : 'NOT_RECEIVED');
    return;
  }
  if (e.data.toString() == '[object VideoFrame]') {
    received.set(e.data.timestamp, e.data);
  }
};
