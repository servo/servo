// Receives a MessagePort from the window and immediately transfers it back.
// This proves a port can pass through a foreign agent cluster without losing
// the ability to participate in shared-memory cloning back in its origin
// cluster.
onconnect = initialE => {
  const port = initialE.source;
  port.onmessage = e => {
    const [bouncedPort] = e.ports;
    port.postMessage("bounced", [bouncedPort]);
  };
};
