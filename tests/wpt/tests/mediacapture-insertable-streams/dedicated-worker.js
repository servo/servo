self.onmessage = (e) => {
  try {
    const mstg = new MediaStreamTrackGenerator({kind: 'video'});
    if ('enable' in e.data) {
      mstg.enabled = e.data.enable;
    }
    self.postMessage({result: 'Success'});
  } catch (e) {
    self.postMessage({result: 'Failure', error: e});
  }
}