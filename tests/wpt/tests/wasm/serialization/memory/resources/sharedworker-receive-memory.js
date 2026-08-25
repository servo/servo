// Receives a shared WebAssembly.Memory from the connecting window and reports
// what it observes. The window expects "got-messageerror" (cross-agent-cluster
// deserialization failure); any other outcome is surfaced verbatim so the test
// fails with diagnostic detail instead of timing out.
onconnect = initialE => {
  const port = initialE.source;
  port.onmessageerror = () => {
    port.postMessage({ kind: "got-messageerror" });
  };
  port.onmessage = e => {
    let typeName;
    if (e.data && typeof e.data === "object" && e.data.constructor) {
      typeName = e.data.constructor.name;
    } else {
      typeName = typeof e.data;
    }
    port.postMessage({ kind: "got-message", type: typeName });
  };
  port.postMessage({ kind: "ready", crossOriginIsolated: self.crossOriginIsolated });
};
