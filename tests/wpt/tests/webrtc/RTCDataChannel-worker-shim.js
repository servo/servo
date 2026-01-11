// This can be used to transfer an RTCDataChannel to a worker, and expose an
// interface that acts as a passthrough to the channel on the worker. There are
// some caveats though:
// * certain kinds of error won't propagate back to the callsite, and will
//  manifest as an unhandled error (eg; webidl errors on attribute setters)
// * the event handler/GC interactions won't translate exactly (because the
//  worker code registers all handler types up front, instead of waiting for
//  registrations to happen on the wrapper)
// * RTCDataChannel.label must be unique on the worker
class WorkerBackedDataChannel extends EventTarget {
  #worker;
  #dcAttrs;
  #eventHandlers;
  #errorPromise;
  #label;

  // If you want to make multiple of these with the same worker, create first
  // with no args, and the others with first.worker
  constructor(worker = null) {
    super();
    this.#worker = worker || WorkerBackedDataChannel.makeWorker();

    // Cache of the RTTCDataChannel's state attributes, filled on init. Some are
    // updated by state updates later.
    this.#dcAttrs = null;

    // For tracking the onxxxx-style event callbacks
    // TODO: Maybe there's a simpler way to do this?
    this.#eventHandlers = new Map();

    this.#listenForEventMessages();

    // Ejection seat that we put in our promises, for cases where we've misused
    // the worker (or its code), or encountered some sort of unhandled error
    this.#errorPromise = new Promise((_, reject) => {
      // the Worker 'error' and 'messageerror' events
      const onErrorEvent = (e) => {
        switch (e.type) {
          case 'error':
          case 'messageerror':
            reject(new Error(`Worker sent ${e.type} event: ${e.message}`));
            break;
        }
      };
      this.#worker.addEventListener('error', onErrorEvent);
      this.#worker.addEventListener('messageerror', onErrorEvent);

      // Unhandled exceptions thrown by *our* worker code; not Worker error
      // events (those are handled above), and not errors thrown by
      // RTCDataChannel (those are handled in #sendRequestToWorker)
      this.#worker.addEventListener('message', ({data}) => {
        const {type, label, result} = data;
        if (type == 'workerError' &&
            (label === undefined || label == this.#label)) {
          reject(new Error(
            `Worker code sent error message: ${result}`));
        }
      });
    });
  }

  async init(channel) {
    this.#label = channel.label;

    // DO NOT GO ASYNC BEFORE THIS! Doing so will render the channel
    // untransferable.
    const initPromise = this.#sendRequestToWorker('init', channel, [channel]);
    this.#dcAttrs = await Promise.race([initPromise, this.#errorPromise]);
    return this.#dcAttrs;
  }

  static makeWorker() {
    return new Worker('/webrtc/RTCDataChannel-worker.js');
  }

  // Make it easy to put more channels on this worker
  get worker() { return this.#worker; }

  // Read-only attributes
  get label() { return this.#dcAttrs.label; }
  get ordered() { return this.#dcAttrs.ordered; }
  get maxPacketLifeTime() { return this.#dcAttrs.maxPacketLifeTime; }
  get maxRetransmits() { return this.#dcAttrs.maxRetransmits; }
  get protocol() { return this.#dcAttrs.protocol; }
  get negotiated() { return this.#dcAttrs.negotiated; }
  get id() { return this.#dcAttrs.id; }
  get readyState() { return this.#dcAttrs.readyState; }
  get bufferedAmount() { return this.#dcAttrs.bufferedAmount; }

  // Writable attributes
  set bufferedAmountLowThreshold(val) {
    this.#dcAttrs.bufferedAmountLowThreshold = val;
    this.#sendRequestToWorker('setBufferedAmountLowThreshold', val);
  }
  get bufferedAmountLowThreshold() {
    return this.#dcAttrs.bufferedAmountLowThreshold;
  }

  set binaryType(val) {
    this.#dcAttrs.binaryType = val;
    this.#sendRequestToWorker('setBinaryType', val);
  }
  get binaryType() {
    return this.#dcAttrs.binaryType;
  }

  // Note: these do not try to match the way the handler is registered on the
  // other end (eg; dc.onopen = handler is performed on the worker as an
  // addEventListener call, not as workerDc.onopen = func). This means that
  // this wrapper is not suitable for testing GC logic based on event handlers.
  set onopen(fn) { this.#setEventHandler('open', fn); }
  set onbufferedamountlow(fn) { this.#setEventHandler('bufferedamountlow', fn); }
  set onerror(fn) { this.#setEventHandler('error', fn); }
  set onclosing(fn) { this.#setEventHandler('closing', fn); }
  set onclose(fn) { this.#setEventHandler('close', fn); }
  set onmessage(fn) { this.#setEventHandler('message', fn); }

  async send(data) {
    return this.#sendRequestToWorker('send', data);
  }

  async close() {
    return this.#sendRequestToWorker('close');
  }

  // Used to refresh readyState, bufferedAmount, and id
  async updateState() {
    const resp = await Promise.race([this.#sendRequestToWorker('queryState'), this.#errorPromise]);
    this.#dcAttrs.readyState = resp.readyState;
    this.#dcAttrs.bufferedAmount = resp.bufferedAmount;
    this.#dcAttrs.id = resp.id;
    return resp;
  }

  #setEventHandler(type, handler) {
    // Listener might not exist, removeEventListener doesn't care
    this.removeEventListener(type, this.#eventHandlers.get(type));
    this.#eventHandlers.delete(type);
    if (handler) {
      this.addEventListener(type, handler);
      this.#eventHandlers.set(type, handler);
    }
  }

  #listenForEventMessages() {
    this.#worker.addEventListener('message', ({data}) => {
      const { type, label, result } = data;

      const eventTypes =
        ['open', 'bufferedamountlow', 'error', 'closing', 'close', 'message'];

      if (label == this.#label && eventTypes.includes(type)) {
        let e;
        if (type == 'message') {
          const {data, origin} = result;
          e = new MessageEvent(type, {data, origin});
        } else {
          e = new Event(type);
        }
        this.dispatchEvent(e);
      }
    });
  }


  #sendRequestToWorker(type, arg, transferOrOptions) {
    if (!this.#label) {
      throw new Error('RTCDataChannel worker shim not initialized!');
    }

    return new Promise((resolve, reject) => {
      // We currently assume that if multiple requests of the same type are
      // sent, they get responses in the same order. That probably won't
      // change, but if it does we'll need a transaction id.
      const msg = { type, label: this.#label, arg };
      const responseType = `${type}Response`

      const onResponse = ({data}) => {
        const {type, label, result} = data;
        if (type == responseType && label == this.#label) {
          this.#worker.removeEventListener('message', onResponse);
          if (result?.error) {
            // Error thrown by RTCDataChannel, other error cases are handled by
            // the code in this.#errorPromise
            // TODO: Maybe re-synthesize the specific error thrown by the
            // RTCDataChannel?
            reject(new Error(`RTCDataChannel error: ${result.error.message}`));
          } else {
            resolve(result);
          }
        }
      };

      this.#worker.addEventListener('message', onResponse);
      this.#worker.postMessage(msg, transferOrOptions);
    });
  }
}
