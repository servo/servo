/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { LogMessageWithStack } from '../../framework/logging/log_message.js';
export class TestWorker {
  constructor(debug) {
    _defineProperty(this, "debug", void 0);

    _defineProperty(this, "worker", void 0);

    _defineProperty(this, "resolvers", new Map());

    this.debug = debug;
    const selfPath = import.meta.url;
    const selfPathDir = selfPath.substring(0, selfPath.lastIndexOf('/'));
    const workerPath = selfPathDir + '/test_worker-worker.js';
    this.worker = new Worker(workerPath, {
      type: 'module'
    });

    this.worker.onmessage = ev => {
      const query = ev.data.query;
      const result = ev.data.result;

      if (result.logs) {
        for (const l of result.logs) {
          Object.setPrototypeOf(l, LogMessageWithStack.prototype);
        }
      }

      this.resolvers.get(query)(result); // TODO(kainino0x): update the Logger with this result (or don't have a logger and update the
      // entire results JSON somehow at some point).
    };
  }

  async run(rec, query) {
    this.worker.postMessage({
      query,
      debug: this.debug
    });
    const workerResult = await new Promise(resolve => {
      this.resolvers.set(query, resolve);
    });
    rec.injectResult(workerResult);
  }

}
//# sourceMappingURL=test_worker.js.map