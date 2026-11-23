const wasmModule = new WebAssembly.Module(new Uint8Array([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]));

onconnect = ({ ports: [control] }) => {
  control.onmessage = ({ data }) => {
    new BroadcastChannel(data).postMessage(wasmModule);
  };
};
