/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

export const description = `
error scope validation tests.
`;
import { Fixture } from '../../../common/framework/fixture.js';
import { getGPU } from '../../../common/framework/gpu/implementation.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { assert, raceWithRejectOnTimeout } from '../../../common/framework/util/util.js';

class F extends Fixture {
  constructor(...args) {
    super(...args);

    _defineProperty(this, "_device", undefined);
  }

  get device() {
    assert(this._device !== undefined);
    return this._device;
  }

  async init() {
    super.init();
    const gpu = getGPU();
    const adapter = await gpu.requestAdapter();
    this._device = await adapter.requestDevice();
  }

  createErrorBuffer() {
    this.device.createBuffer({
      size: 1024,
      usage: 0xffff // Invalid GPUBufferUsage

    }); // TODO: Remove when chrome does it automatically.

    this.device.defaultQueue.submit([]);
  } // Expect an uncapturederror event to occur. Note: this MUST be awaited, because
  // otherwise it could erroneously pass by capturing an error from later in the test.


  async expectUncapturedError(fn) {
    return this.immediateAsyncExpectation(() => {
      // TODO: Make arbitrary timeout value a test runner variable
      const TIMEOUT_IN_MS = 1000;
      const promise = new Promise(resolve => {
        const eventListener = event => {
          this.debug(`Got uncaptured error event with ${event.error}`);
          resolve(event);
        };

        this.device.addEventListener('uncapturederror', eventListener, {
          once: true
        });
      });
      fn();
      return raceWithRejectOnTimeout(promise, TIMEOUT_IN_MS, 'Timeout occurred waiting for uncaptured error');
    });
  }

}

export const g = makeTestGroup(F);
g.test('simple_case_where_the_error_scope_catches_an_error').fn(async t => {
  t.device.pushErrorScope('validation');
  t.createErrorBuffer();
  const error = await t.device.popErrorScope();
  t.expect(error instanceof GPUValidationError);
});
g.test('errors_bubble_to_the_parent_scope_if_not_handled_by_the_current_scope').fn(async t => {
  t.device.pushErrorScope('validation');
  t.device.pushErrorScope('out-of-memory');
  t.createErrorBuffer();
  {
    const error = await t.device.popErrorScope();
    t.expect(error === null);
  }
  {
    const error = await t.device.popErrorScope();
    t.expect(error instanceof GPUValidationError);
  }
});
g.test('if_an_error_scope_matches_an_error_it_does_not_bubble_to_the_parent_scope').fn(async t => {
  t.device.pushErrorScope('validation');
  t.device.pushErrorScope('validation');
  t.createErrorBuffer();
  {
    const error = await t.device.popErrorScope();
    t.expect(error instanceof GPUValidationError);
  }
  {
    const error = await t.device.popErrorScope();
    t.expect(error === null);
  }
});
g.test('if_no_error_scope_handles_an_error_it_fires_an_uncapturederror_event').fn(async t => {
  t.device.pushErrorScope('out-of-memory');
  const uncapturedErrorEvent = await t.expectUncapturedError(() => {
    t.createErrorBuffer();
  });
  t.expect(uncapturedErrorEvent.error instanceof GPUValidationError);
  const error = await t.device.popErrorScope();
  t.expect(error === null);
});
g.test('push,popping_sibling_error_scopes_must_be_balanced').fn(async t => {
  {
    const promise = t.device.popErrorScope();
    t.shouldReject('OperationError', promise);
  }
  const promises = [];

  for (let i = 0; i < 1000; i++) {
    t.device.pushErrorScope('validation');
    promises.push(t.device.popErrorScope());
  }

  const errors = await Promise.all(promises);
  t.expect(errors.every(e => e === null));
  {
    const promise = t.device.popErrorScope();
    t.shouldReject('OperationError', promise);
  }
});
g.test('push,popping_nested_error_scopes_must_be_balanced').fn(async t => {
  {
    const promise = t.device.popErrorScope();
    t.shouldReject('OperationError', promise);
  }
  const promises = [];

  for (let i = 0; i < 1000; i++) {
    t.device.pushErrorScope('validation');
  }

  for (let i = 0; i < 1000; i++) {
    promises.push(t.device.popErrorScope());
  }

  const errors = await Promise.all(promises);
  t.expect(errors.every(e => e === null));
  {
    const promise = t.device.popErrorScope();
    t.shouldReject('OperationError', promise);
  }
});
//# sourceMappingURL=error_scope.spec.js.map