onconnect = (e) => {
    const port = e.ports[0];
    port.onmessage = (e) => {
      try {
        const generator = new MediaStreamTrackGenerator({kind: 'video'});
        port.postMessage({result: 'Success'});
      } catch (e) {
        port.postMessage({result: 'Failure', error: e});
      }
    }
}