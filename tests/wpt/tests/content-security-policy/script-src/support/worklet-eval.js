class EvalProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.port.onmessage = (event) => {
      try {
        eval("1 + 2 + 3");
        this.port.postMessage("eval allowed");
      } catch (e) {
        this.port.postMessage("eval blocked");
      }
    };
  }
  process() { return false; }
}
registerProcessor('eval-processor', EvalProcessor);
