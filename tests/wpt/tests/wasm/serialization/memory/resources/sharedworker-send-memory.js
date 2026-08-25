// Posts a shared WebAssembly.Memory to the connecting window when asked. Any
// synchronous failure during postMessage is reported verbatim so the test
// surfaces a clear failure rather than timing out.
onconnect = initialE => {
  const port = initialE.source;
  port.onmessage = e => {
    if (e.data === "send-me-memory") {
      try {
        port.postMessage(new WebAssembly.Memory({ shared: true, initial: 1, maximum: 1 }));
      } catch (err) {
        port.postMessage({ kind: "threw", message: String(err) });
      }
    } else {
      port.postMessage({ kind: "unexpected-message", value: e.data });
    }
  };
  port.postMessage({ kind: "ready", crossOriginIsolated: self.crossOriginIsolated });
};
