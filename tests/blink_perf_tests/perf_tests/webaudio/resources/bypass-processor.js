/**
 * @class BypassProcessor
 * @extends AudioWorkletProcessor
 */
class BypassProcessor extends AudioWorkletProcessor {
  process(inputs, outputs) {
    let input = inputs[0];
    let output = outputs[0];
    for (let channel = 0; channel < input.length; ++channel)
      output[channel].set(input[channel]);

    return true;
  }
}

registerProcessor('bypass-processor', BypassProcessor);