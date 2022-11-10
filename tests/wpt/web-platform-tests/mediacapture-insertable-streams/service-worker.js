self.addEventListener('message', (event) => {
  try {
    const mstg = new MediaStreamTrackGenerator({ kind: 'video' });
    event.source.postMessage({ result: 'Success' });
  } catch (e) {
    event.source.postMessage({ result: 'Failure', error: e });
  };
});