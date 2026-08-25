/**
 * @class RenderSizeHintProcessor
 * @extends AudioWorkletProcessor
 *
 * This processor class is used to verify that renderQuantumSize propagates
 * correctly to the AudioWorkletGlobalScope and that the process() method
 * receives buffers of the requested render quantum size.
 */
class RenderSizeHintProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this._processMessageSent = false;

    // Post the renderQuantumSize from AudioWorkletGlobalScope immediately
    // upon construction.
    this.port.postMessage({
      type: 'constructor',
      renderQuantumSize: renderQuantumSize
    });
  }

  process(inputs, outputs) {
    if (!this._processMessageSent) {
      // Verify the actual buffer length passed to process().
      // Use outputs[0][0].length as a safe fallback if inputs are
      // unconnected.
      const bufferLength = (inputs[0] && inputs[0][0])
          ? inputs[0][0].length
          : outputs[0][0].length;
      this.port.postMessage({
        type: 'process',
        length: bufferLength
      });
      this._processMessageSent = true;
    }
    return false;
  }
}

registerProcessor('rendersizehint-processor', RenderSizeHintProcessor);
