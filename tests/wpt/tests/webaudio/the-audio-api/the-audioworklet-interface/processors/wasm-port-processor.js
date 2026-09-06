function handleWasmModule(messagePort, event) {
  new WebAssembly.Instance(event.data);
  messagePort.postMessage({type: 'wasm-module-received', module: event.data});
}

function reportWasmModuleMessageError(messagePort) {
  messagePort.postMessage({type: 'wasm-module-messageerror'});
}

/**
 * @class WasmPortProcessor
 * @extends AudioWorkletProcessor
 *
 * This processor class demonstrates WebAssembly.Module messaging.
 */
class WasmPortProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.port.onmessage = (event) => handleWasmModule(this.port, event);
    this.port.onmessageerror = () => reportWasmModuleMessageError(this.port);
  }

  process() {
    return true;
  }
}

registerProcessor('wasm-port-processor', WasmPortProcessor);

port.onmessage = (event) => handleWasmModule(port, event);
port.onmessageerror = () => reportWasmModuleMessageError(port);
