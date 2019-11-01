/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

export class TestWorker {
  constructor() {
    _defineProperty(this, "worker", void 0);

    _defineProperty(this, "resolvers", new Map());

    const selfPath = import.meta.url;
    const selfPathDir = selfPath.substring(0, selfPath.lastIndexOf('/'));
    const workerPath = selfPathDir + '/test_worker.worker.js';
    this.worker = new Worker(workerPath, {
      type: 'module'
    });

    this.worker.onmessage = ev => {
      const {
        query,
        result
      } = ev.data;
      this.resolvers.get(query)(result); // TODO(kainino0x): update the Logger with this result (or don't have a logger and update the
      // entire results JSON somehow at some point).
    };
  }

  run(query, debug = false) {
    this.worker.postMessage({
      query,
      debug
    });
    return new Promise(resolve => {
      this.resolvers.set(query, resolve);
    });
  }

}
//# sourceMappingURL=test_worker.js.map