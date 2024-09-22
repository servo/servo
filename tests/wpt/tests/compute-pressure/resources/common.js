'use strict';

// Dependencies:
// * /common/utils.js
// * /common/dispatcher/dispatcher.js
//
// This file contains the required infrastructure to run Compute Pressure tests
// in both window and worker scopes transparently.
//
// We cannot just use the '.any.js' mechanism with "META: global=window,etc"
// because we need the worker needs to manipulate virtual pressure sources, and
// we can only do that by posting a message to the embedder window to do it due
// to testdriver's limitations and operation model.
//
// See README.md for how to use pressure_test() and other functions within this
// file.
//
// Example:
// - /compute-pressure/foo.https.window.js
//   // META: variant=?globalScope=window
//   // META: variant=?globalScope=dedicated_worker
//   // META: script=/resources/testdriver.js
//   // META: script=/resources/testdriver-vendor.js
//   // META: script=/common/utils.js
//   // META: script=/common/dispatcher/dispatcher.js
//   // META: script=./resources/common.js
//
//   pressure_test(async t => {
//     await create_virtual_pressure_source("cpu");
//     t.add_cleanup(async () => {
//       await remove_virtual_pressure_source("cpu")
//     });
//     /* rest of the test */
//   }, "my test");
//
//   pressure_test(async t => { /* ... */ });
//
//   mark_as_done();

class WindowHelper {
  constructor() {
    setup({explicit_done: true});

    // These are the calls made by tests that use pressure_test(). We do not
    // invoke the actual virtual pressure functions directly because of
    // compatibility with the dedicated workers case, where these calls are
    // done via postMessage (see resources/worker-support.js).
    globalThis.create_virtual_pressure_source =
        test_driver.create_virtual_pressure_source.bind(test_driver);
    globalThis.remove_virtual_pressure_source =
        test_driver.remove_virtual_pressure_source.bind(test_driver);
    globalThis.update_virtual_pressure_source =
        test_driver.update_virtual_pressure_source.bind(test_driver);
  }

  mark_as_done() {
    done();
  }

  pressure_test(test_func, description) {
    promise_test(test_func, description);
  }
}

class DedicatedWorkerHelper {
  constructor() {
    this.token = token();

    this.worker = new Worker(
        `/compute-pressure/resources/worker-support.js?uuid=${this.token}`);
    this.worker.onmessage = async (e) => {
      if (!e.data.command) {
        return;
      }

      switch (e.data.command) {
        case 'create':
          await test_driver.create_virtual_pressure_source(...e.data.params);
          break;

        case 'remove':
          await test_driver.remove_virtual_pressure_source(...e.data.params);
          break;

        case 'update':
          await test_driver.update_virtual_pressure_source(...e.data.params);
          break;

        default:
          throw new Error(`Unexpected command '${e.data.command}'`);
      }

      this.worker.postMessage({
        command: e.data.command,
        id: e.data.id,
      });
    };

    // We need to call this here so that the testharness RemoteContext
    // infrastructure is set up before pressure_test() is called, as each test
    // will be added after the worker scaffolding code has loaded.
    this.fetch_tests_promise = fetch_tests_from_worker(this.worker);

    this.ctx = new RemoteContext(this.token);

    this.pending_tests = [];
  }

  async mark_as_done() {
    await Promise.all(this.pending_tests);
    await this.ctx.execute_script(() => {
      done();
    });
    await this.fetch_tests_from_worker;
  }

  async pressure_test(test_func, description) {
    this.pending_tests.push(this.ctx.execute_script(
        `
        (description) => promise_test(${test_func}, description);
      `,
        [description]));
  }
}

let _pressureTestHelper;
const _globalScope = new URLSearchParams(location.search).get('globalScope');
switch (_globalScope) {
  case 'window':
    _pressureTestHelper = new WindowHelper();
    break;
  case 'dedicated_worker':
    _pressureTestHelper = new DedicatedWorkerHelper();
    break;
  default:
    throw new Error(`Invalid variant '${_globalScope}'`);
}

const pressure_test =
    _pressureTestHelper.pressure_test.bind(_pressureTestHelper);
const mark_as_done = _pressureTestHelper.mark_as_done.bind(_pressureTestHelper);
