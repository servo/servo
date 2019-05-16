/*
 * @class AddOffsetProcessor
 * @extends AudioWorkletProcessor
 *
 * Just adds a fixed value to the input
 */
class AddOffsetProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super();

    this._offset = options.processorOptions.offset;
  }

  process(inputs, outputs, parameters) {
    let input = inputs[0][0];
    let output = outputs[0][0];
    if (input.length > 0) {
      for (let k = 0; k < input.length; ++k) {
        output[k] = input[k] + this._offset;
      }
    } else {
      // No input connected, so pretend it's silence and just fill the
      // output with the offset value.
      output.fill(this._offset);
    }

    return true;
  }
}

registerProcessor('add-offset-processor', AddOffsetProcessor);
