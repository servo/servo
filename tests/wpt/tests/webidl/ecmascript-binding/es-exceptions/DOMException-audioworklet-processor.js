class DOMExceptionProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.port.onmessage = e => this.port.postMessage({
      type: 'message',
      isDOMException: e.data instanceof DOMException,
      isQuotaExceededError: typeof QuotaExceededError === 'function' && e.data instanceof QuotaExceededError,
      name: e.data?.name,
      message: e.data?.message,
    });
    this.port.onmessageerror = () => this.port.postMessage({ type: 'messageerror' });

    let created;
    try {
      created = {
        domException: new DOMException('created in a worklet', 'SyntaxError'),
        quotaExceededError: new QuotaExceededError('also created in a worklet', { quota: 1, requested: 2 }),
      };
    } catch (e) {
      created = { error: String(e) };
    }
    this.port.postMessage({
      type: 'ready',
      typeofDOMException: typeof DOMException,
      typeofQuotaExceededError: typeof QuotaExceededError,
      created,
    });
  }

  process() {
    return false;
  }
}

registerProcessor('domexception-processor', DOMExceptionProcessor);
