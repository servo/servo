/**
 * @class ConstructorErrorProcessor
 * @extends AudioWorkletProcessor
 */
class ConstructorErrorProcessor extends AudioWorkletProcessor {
  constructor() {
    throw 'ConstructorErrorProcessor: an error thrown from constructor.';
  }

  process() {
    return true;
  }
}


/**
 * @class ProcessErrorProcessor
 * @extends AudioWorkletProcessor
 */
class ProcessErrorProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
  }

  process() {
    throw 'ProcessErrorProcessor: an error throw from process method.';
    return true;
  }
}


registerProcessor('constructor-error', ConstructorErrorProcessor);
registerProcessor('process-error', ProcessErrorProcessor);
