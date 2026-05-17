// Spawns a dedicated worker on first message from the window, forwards the
// transferred MessagePort to it, and proxies status messages back to the
// window. The dedicated worker lives in this SharedWorker's agent cluster
// (different from the window's), so shared-memory cloning to its port should
// fail at deserialization with a messageerror event in the dedicated worker.
let dedicatedWorker;
let controlPort;

onconnect = initialE => {
  controlPort = initialE.source;
  controlPort.onmessage = e => {
    const portForDedicated = e.ports[0];
    dedicatedWorker = new Worker("dedicated-worker-port-failure.js");
    dedicatedWorker.onmessage = ev => {
      controlPort.postMessage(ev.data);
    };
    dedicatedWorker.postMessage(null, [portForDedicated]);
  };
};
