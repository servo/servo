/**
 * @class UndefinedProcessProcessor
 * @extends AudioWorkletProcessor
 *
 * This processor class intentionally does not have a process() function.
 */
class UndefinedProcessProcessor extends AudioWorkletProcessor {}

/**
 * @class NonCallableDataProcessProcessor
 * @extends AudioWorkletProcessor
 *
 * A processor whose process is an own data property that is not callable.
 */
class NonCallableDataProcessProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.process = 42;
  }
}

/**
 * @class NonCallableGetterProcessProcessor
 * @extends AudioWorkletProcessor
 *
 * A processor whose process getter returns a non-callable value.
 */
class NonCallableGetterProcessProcessor extends AudioWorkletProcessor {
  get process() {
    return 42;
  }
}

/**
 * @class ThrowingGetterProcessProcessor
 * @extends AudioWorkletProcessor
 *
 * A processor whose process getter throws an Error.
 */
class ThrowingGetterProcessProcessor extends AudioWorkletProcessor {
  get process() {
    throw new Error('process getter error');
  }
}

/**
 * @class LateBoundProcessProcessor
 * @extends AudioWorkletProcessor
 *
 * A processor without process at construction that assigns it upon receiving
 * a message, before rendering starts.
 */
class LateBoundProcessProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.port.onmessage = () => {
      this.process = (inputs, outputs) => {
        for (const channel of outputs[0]) {
          channel.fill(1);
        }
        return true;
      };
      this.port.postMessage('bound');
    };
  }
}

registerProcessor('missing-process', UndefinedProcessProcessor);
registerProcessor('non-callable-data-process', NonCallableDataProcessProcessor);
registerProcessor('non-callable-getter-process',
                  NonCallableGetterProcessProcessor);
registerProcessor('throwing-getter-process', ThrowingGetterProcessProcessor);
registerProcessor('late-bound-process', LateBoundProcessProcessor);
