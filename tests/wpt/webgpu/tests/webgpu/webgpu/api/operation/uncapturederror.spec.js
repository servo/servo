/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for GPUDevice.onuncapturederror / addEventListener('uncapturederror')
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { raceWithRejectOnTimeout } from '../../../common/util/util.js';
import { kGeneratableErrorScopeFilters } from '../../capability_info.js';
import { ErrorTest } from '../../error_test.js';

export const g = makeTestGroup(ErrorTest);

g.test('iff_uncaptured').
desc(
  `{validation, out-of-memory} error should fire uncapturederror iff not captured by a scope.`
).
params((u) =>
u.
combine('useOnuncapturederror', [false, true]).
combine('errorType', kGeneratableErrorScopeFilters)
).
fn(async (t) => {
  const { useOnuncapturederror, errorType } = t.params;
  const uncapturedErrorEvent = await t.expectUncapturedError(() => {
    t.generateError(errorType);
  }, useOnuncapturederror);
  t.expect(t.isInstanceOfError(errorType, uncapturedErrorEvent.error));
});

g.test('only_original_device_is_event_target').
desc(
  `Original GPUDevice objects are EventTargets and have onuncapturederror, but
deserialized GPUDevices do not.`
).
unimplemented();

g.test('uncapturederror_from_non_originating_thread').
desc(
  `Uncaptured errors on any thread should always propagate to the original GPUDevice object
(since deserialized ones don't have EventTarget/onuncapturederror).`
).
unimplemented();

g.test('onuncapturederror_order_wrt_addEventListener').
desc(
  `
Test that onuncapturederror and addEventListener work in the correct order.

The spec says setting onuncapturederror adds a listener via addEventListener that
calls the callback. Changing onuncapturederror changes the callback to the existing
listener. Setting onuncapturederror to null removes the listener.
  `
).
fn(async (t) => {
  const callOrder = [];

  const makeListener = (id) => {
    let resolve;
    return {
      getPromise() {
        return new Promise((r) => {
          resolve = r;
        });
      },
      listener: (e) => {
        e.preventDefault();
        t.debug(`listener ${id} called`);
        callOrder.push(id);
        resolve();
      }
    };
  };

  const listenerA = makeListener('a');
  const listenerB = makeListener('b');
  const callbackC = makeListener('c');
  const callbackD = makeListener('d');
  const callbackE = makeListener('e');

  try {
    t.debug('test they are called in the order added');
    {
      const promises = [listenerA.getPromise(), listenerB.getPromise(), callbackC.getPromise()];

      t.device.addEventListener('uncapturederror', listenerA.listener);
      t.device.onuncapturederror = callbackC.listener;
      t.device.addEventListener('uncapturederror', listenerB.listener);

      t.generateError('validation');
      await raceWithRejectOnTimeout(Promise.all(promises), 500, 'timeout1');

      const order = callOrder.join(',');
      t.expect(order === 'a,c,b', `'${order}' === 'a,c,b'`);
      callOrder.length = 0;
    }

    t.debug('test changing onuncapturederror does not change the order');
    {
      const promises = [listenerA.getPromise(), listenerB.getPromise(), callbackD.getPromise()];

      t.device.onuncapturederror = callbackD.listener;

      t.generateError('validation');
      await raceWithRejectOnTimeout(Promise.all(promises), 500, 'timeout2');

      const order = callOrder.join(',');
      t.expect(order === 'a,d,b', `'${order}' === 'a,d,b'`);
      callOrder.length = 0;
    }

    t.debug('test clearing onuncapturederror then setting it does change the order');
    {
      const promises = [listenerA.getPromise(), listenerB.getPromise(), callbackE.getPromise()];

      t.device.onuncapturederror = null;
      t.device.onuncapturederror = callbackE.listener;

      t.generateError('validation');
      await raceWithRejectOnTimeout(Promise.all(promises), 500, 'timeout3');

      const order = callOrder.join(',');
      t.expect(order === 'a,b,e', `'${order}' === 'a,b,e'`);
      callOrder.length = 0;
    }
  } finally {
    t.device.onuncapturederror = null;
    t.device.removeEventListener('uncapturederror', listenerA.listener);
    t.device.removeEventListener('uncapturederror', listenerB.listener);
  }
});