/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { LogMessageWithStack } from '../../internal/logging/log_message.js';


import { kDefaultCTSOptions } from './options.js';

export class TestWorker {


  resolvers = new Map();

  constructor(ctsOptions) {
    this.ctsOptions = { ...(ctsOptions || kDefaultCTSOptions), ...{ worker: true } };
    const selfPath = import.meta.url;
    const selfPathDir = selfPath.substring(0, selfPath.lastIndexOf('/'));
    const workerPath = selfPathDir + '/test_worker-worker.js';
    this.worker = new Worker(workerPath, { type: 'module' });
    this.worker.onmessage = (ev) => {
      const query = ev.data.query;
      const result = ev.data.result;
      if (result.logs) {
        for (const l of result.logs) {
          Object.setPrototypeOf(l, LogMessageWithStack.prototype);
        }
      }
      this.resolvers.get(query)(result);

      // MAINTENANCE_TODO(kainino0x): update the Logger with this result (or don't have a logger and
      // update the entire results JSON somehow at some point).
    };
  }

  async run(
  rec,
  query,
  expectations = [])
  {
    this.worker.postMessage({
      query,
      expectations,
      ctsOptions: this.ctsOptions
    });
    const workerResult = await new Promise((resolve) => {
      this.resolvers.set(query, resolve);
    });
    rec.injectResult(workerResult);
  }
}