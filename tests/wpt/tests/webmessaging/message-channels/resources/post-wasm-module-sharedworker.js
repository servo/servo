const wasmModule = new WebAssembly.Module(new Uint8Array([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]));

onconnect = ({ ports: [control] }) => {
  let port;
  control.onmessage = ({ data, ports }) => {
    if (ports.length) {
      port = ports[0];
      control.postMessage("received");
      return;
    }
    if (data === "send")
      port.postMessage(wasmModule);
  };
};
