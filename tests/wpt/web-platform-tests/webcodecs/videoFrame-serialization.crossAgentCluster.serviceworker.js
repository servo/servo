const data = new Uint8Array([
  1, 2, 3, 4, 5, 6, 7, 8,
  9, 10, 11, 12, 13, 14, 15, 16,
]);
let received = new Map();
self.onmessage = (e) => {
  if (e.data == 'create-frame') {
    let frameOrError = null;
    try {
      frameOrError = new VideoFrame(data, {
        timestamp: 0,
        codedWidth: 2,
        codedHeight: 2,
        format: 'RGBA',
      });
    } catch (error) {
      frameOrError = error
    }
    e.source.postMessage(frameOrError);
    return;
  }
  if (e.data.hasOwnProperty('id')) {
    e.source.postMessage(
      received.get(e.data.id) ? 'RECEIVED' : 'NOT_RECEIVED');
    return;
  }
  if (e.data.toString() == '[object VideoFrame]') {
    received.set(e.data.timestamp, e.data);
  }
};
