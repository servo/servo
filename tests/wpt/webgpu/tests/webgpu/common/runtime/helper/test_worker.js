/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { LogMessageWithStack } from '../../internal/logging/log_message.js';

import { timeout } from '../../util/timeout.js';
import { assert } from '../../util/util.js';

import { kDefaultCTSOptions } from './options.js';


/** Query all currently-registered service workers, and unregister them. */
function unregisterAllServiceWorkers() {
  void navigator.serviceWorker.getRegistrations().then((registrations) => {
    for (const registration of registrations) {
      void registration.unregister();
    }
  });
}

// Firefox has serviceWorkers disabled in private mode
// and Servo does not support serviceWorkers yet.
if ('serviceWorker' in navigator) {
  // NOTE: This code runs on startup for any runtime with worker support. Here, we use that chance to
  // delete any leaked service workers, and register to clean up after ourselves at shutdown.
  unregisterAllServiceWorkers();
  window.addEventListener('beforeunload', () => {
    unregisterAllServiceWorkers();
  });
}

class TestBaseWorker {

  resolvers = new Map();

  constructor(worker, ctsOptions) {
    this.ctsOptions = { ...(ctsOptions || kDefaultCTSOptions), ...{ worker } };
  }

  onmessage(ev) {
    const query = ev.data.query;
    const transferredResult = ev.data.result;

    const result = {
      status: transferredResult.status,
      timems: transferredResult.timems,
      logs: transferredResult.logs?.map((l) => new LogMessageWithStack(l))
    };

    this.resolvers.get(query)(result);
    this.resolvers.delete(query);

    // MAINTENANCE_TODO(kainino0x): update the Logger with this result (or don't have a logger and
    // update the entire results JSON somehow at some point).
  }

  makeRequestAndRecordResult(
  target,
  query,
  expectations)
  {
    const request = {
      query,
      expectations,
      ctsOptions: this.ctsOptions
    };
    target.postMessage(request);

    return new Promise((resolve) => {
      assert(!this.resolvers.has(query), "can't request same query twice simultaneously");
      this.resolvers.set(query, resolve);
    });
  }

  async run(
  rec,
  query,
  expectations = [])
  {
    try {
      rec.injectResult(await this.runImpl(query, expectations));
    } catch (ex) {
      rec.start();
      rec.threw(ex);
      rec.finish();
    }
  }





}

export class TestDedicatedWorker extends TestBaseWorker {


  constructor(ctsOptions) {
    super('dedicated', ctsOptions);
    try {
      if (typeof Worker === 'undefined') {
        throw new Error('Dedicated Workers not available');
      }

      const selfPath = import.meta.url;
      const selfPathDir = selfPath.substring(0, selfPath.lastIndexOf('/'));
      const workerPath = selfPathDir + '/test_worker-worker.js';
      this.worker = new Worker(workerPath, { type: 'module' });
      this.worker.onmessage = (ev) => this.onmessage(ev);
    } catch (ex) {
      assert(ex instanceof Error);
      // Save the exception to re-throw in runImpl().
      this.worker = ex;
    }
  }

  runImpl(query, expectations = []) {
    if (this.worker instanceof Worker) {
      return this.makeRequestAndRecordResult(this.worker, query, expectations);
    } else {
      throw this.worker;
    }
  }
}

/** @deprecated Use TestDedicatedWorker instead. */
export class TestWorker extends TestDedicatedWorker {}

export class TestSharedWorker extends TestBaseWorker {
  /** MessagePort to the SharedWorker, or an Error if it couldn't be initialized. */


  constructor(ctsOptions) {
    super('shared', ctsOptions);
    try {
      if (typeof SharedWorker === 'undefined') {
        throw new Error('Shared Workers not available');
      }

      const selfPath = import.meta.url;
      const selfPathDir = selfPath.substring(0, selfPath.lastIndexOf('/'));
      const workerPath = selfPathDir + '/test_worker-worker.js';
      const worker = new SharedWorker(workerPath, { type: 'module' });
      this.port = worker.port;
      this.port.start();
      this.port.onmessage = (ev) => this.onmessage(ev);
    } catch (ex) {
      assert(ex instanceof Error);
      // Save the exception to re-throw in runImpl().
      this.port = ex;
    }
  }

  runImpl(query, expectations = []) {
    if (this.port instanceof MessagePort) {
      return this.makeRequestAndRecordResult(this.port, query, expectations);
    } else {
      throw this.port;
    }
  }
}

export class TestServiceWorker extends TestBaseWorker {
  constructor(ctsOptions) {
    super('service', ctsOptions);
  }

  async runImpl(query, expectations = []) {
    if (!('serviceWorker' in navigator)) {
      throw new Error('Service Workers not available');
    }
    const [suite, name] = query.split(':', 2);
    const fileName = name.split(',').join('/');

    const selfPath = import.meta.url;
    const selfPathDir = selfPath.substring(0, selfPath.lastIndexOf('/'));
    // Construct the path to the worker file, then use URL to resolve the `../` components.
    const serviceWorkerURL = new URL(
      `${selfPathDir}/../../../${suite}/webworker/${fileName}.as_worker.js`
    ).toString();

    // If a registration already exists for this path, it will be ignored.
    const registration = await navigator.serviceWorker.register(serviceWorkerURL, {
      type: 'module'
    });
    // Make sure the registration we just requested is active. (We don't worry about it being
    // outdated from a previous page load, because we wipe all service workers on shutdown/startup.)
    while (!registration.active || registration.active.scriptURL !== serviceWorkerURL) {
      await new Promise((resolve) => timeout(resolve, 0));
    }
    const serviceWorker = registration.active;

    navigator.serviceWorker.onmessage = (ev) => this.onmessage(ev);
    return this.makeRequestAndRecordResult(serviceWorker, query, expectations);
  }
}