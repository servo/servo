import source modSource from './worker.wasm';

class AudioProcessor extends AudioWorkletProcessor {
  constructor(...args) {
    super(...args);
    let port = this.port;

    port.onmessage = (e) => {
      let staticCheck = false;
      let dynamicCheck = false;
      const pm =
          (x) => {
            const message = {
              value: x,
              staticCheck: staticCheck,
              dynamicCheck: dynamicCheck
            };
            port.postMessage(message);
          }

      staticCheck = modSource instanceof WebAssembly.Module;
      // `import.source` should fail because dynamic imports aren't supported
      // in worklets.
      let import_promise = import.source('./execute-start.wasm');
      import_promise
          .catch((e) => {
            dynamicCheck = e instanceof TypeError;
          })
          .then(() => {
            // worker.wasm will call pm with the result, so instantiate this
            // after the dynamic check.
            WebAssembly.instantiate(
                modSource, {'./worker-helper.js': {'pm': pm}});
          });
    };
  }

  process(inputs, outputs, parameters) {
    return true;
  }
}

registerProcessor('audio-processor', AudioProcessor);
