/**
 * @class RecorderProcessor
 * @extends AudioWorkletProcessor
 *
 * A simple recorder AudioWorkletProcessor. Returns the recorded buffer to the
 * node when recording is finished.
 */
class RecorderProcessor extends AudioWorkletProcessor {
  /**
   * @param {*} options
   * @param {number} options.duration A duration to record in seconds.
   * @param {number} options.channelCount A channel count to record.
   */
  constructor(options) {
    super();
    this._createdAt = currentTime;
    this._elapsed = 0;
    this._recordDuration = options.duration || 1;
    this._recordChannelCount = options.channelCount || 1;
    this._recordBufferLength = sampleRate * this._recordDuration;
    this._recordBuffer = [];
    for (let i = 0; i < this._recordChannelCount; ++i) {
      this._recordBuffer[i] = new Float32Array(this._recordBufferLength);
    }
  }

  process(inputs, outputs) {
    if (this._recordBufferLength <= currentFrame) {
      this.port.postMessage({
        type: 'recordfinished',
        recordBuffer: this._recordBuffer
      });
      return false;
    }

    // Records the incoming data from |inputs| and also bypasses the data to
    // |outputs|.
    const input = inputs[0];
    const output = outputs[0];
    for (let channel = 0; channel < input.length; ++channel) {
      const inputChannel = input[channel];
      const outputChannel = output[channel];
      outputChannel.set(inputChannel);

      const buffer = this._recordBuffer[channel];
      const capacity = buffer.length - currentFrame;
      buffer.set(inputChannel.slice(0, capacity), currentFrame);
    }

    return true;
  }
}

registerProcessor('recorder-processor', RecorderProcessor);
