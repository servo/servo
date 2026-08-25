/**
 * @class LifetimeProcessor
 * @extends AudioWorkletProcessor
 *
 * This processor returns false from process() to verify that its lifetime
 * is governed by active input connections when the active source flag is
 * false.
 */
class LifetimeProcessor extends AudioWorkletProcessor {
  process(inputs, outputs) {
    // Write 1.0 to all channels of the first output.
    const output = outputs[0];
    for (let channel = 0; channel < output.length; ++channel) {
      output[channel].fill(1.0);
    }

    // Returning false sets the active source flag to false.
    return false;
  }
}

registerProcessor('lifetime-processor', LifetimeProcessor);
