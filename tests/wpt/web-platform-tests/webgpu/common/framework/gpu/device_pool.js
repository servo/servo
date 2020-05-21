/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { assert, raceWithRejectOnTimeout, unreachable, assertReject } from '../util/util.js';
import { getGPU } from './implementation.js';

class TestFailedButDeviceReusable extends Error {}

export class TestOOMedShouldAttemptGC extends Error {}
const kPopErrorScopeTimeoutMS = 5000;
export class DevicePool {
  constructor() {
    _defineProperty(this, "failed", false);

    _defineProperty(this, "holder", undefined);
  }

  // undefined if "uninitialized" (not yet initialized, or lost)
  async acquire() {
    assert(!this.failed, 'WebGPU device previously failed to initialize; not retrying');

    if (this.holder === undefined) {
      try {
        this.holder = await DevicePool.makeHolder();
      } catch (ex) {
        this.failed = true;
        throw ex;
      }
    }

    assert(!this.holder.acquired, 'Device was in use on DevicePool.acquire');
    this.holder.acquired = true;
    this.beginErrorScopes();
    return this.holder.device;
  } // When a test is done using a device, it's released back into the pool.
  // This waits for error scopes, checks their results, and checks for various error conditions.


  async release(device) {
    const holder = this.holder;
    assert(holder !== undefined, 'trying to release a device while pool is uninitialized');
    assert(holder.acquired, 'trying to release a device while already released');
    assert(device === holder.device, 'Released device was the wrong device');

    try {
      // Time out if popErrorScope never completes. This could happen due to a browser bug - e.g.,
      // as of this writing, on Chrome GPU process crash, popErrorScope just hangs.
      await raceWithRejectOnTimeout(this.endErrorScopes(), kPopErrorScopeTimeoutMS, 'finalization popErrorScope timed out'); // (Hopefully if the device was lost, it has been reported by the time endErrorScopes()
      // has finished (or timed out). If not, it could cause a finite number of extra test
      // failures following this one (but should recover eventually).)

      const lostReason = holder.lostReason;

      if (lostReason !== undefined) {
        // Fail the current test.
        unreachable(`Device was lost: ${lostReason}`);
      }
    } catch (ex) {
      // Any error that isn't explicitly TestFailedButDeviceReusable forces a new device to be
      // created for the next test.
      if (!(ex instanceof TestFailedButDeviceReusable)) {
        this.holder = undefined;
      }

      throw ex;
    } finally {
      // TODO: device.destroy()
      // Mark the holder as free. (This only has an effect if the pool still has the holder.)
      // This could be done at the top but is done here to guard against async-races during release.
      holder.acquired = false;
    }
  } // Gets a device and creates a DeviceHolder.
  // If the device is lost, DeviceHolder.lostReason gets set.


  static async makeHolder() {
    const gpu = getGPU();
    const adapter = await gpu.requestAdapter();
    const holder = {
      acquired: false,
      device: await adapter.requestDevice(),
      lostReason: undefined
    };
    holder.device.lost.then(ev => {
      holder.lostReason = ev.message;
    });
    return holder;
  } // Create error scopes that wrap the entire test.


  beginErrorScopes() {
    assert(this.holder !== undefined);
    this.holder.device.pushErrorScope('out-of-memory');
    this.holder.device.pushErrorScope('validation');
  } // End the whole-test error scopes. Check that there are no extra error scopes, and that no
  // otherwise-uncaptured errors occurred during the test.


  async endErrorScopes() {
    assert(this.holder !== undefined);
    let gpuValidationError;
    let gpuOutOfMemoryError;

    try {
      // May reject if the device was lost.
      gpuValidationError = await this.holder.device.popErrorScope();
      gpuOutOfMemoryError = await this.holder.device.popErrorScope();
    } catch (ex) {
      assert(this.holder.lostReason !== undefined, "popErrorScope failed, but device.lost hasn't fired (yet)");
      throw ex;
    }

    await assertReject(this.holder.device.popErrorScope(), 'There was an extra error scope on the stack after a test');

    if (gpuValidationError !== null) {
      assert(gpuValidationError instanceof GPUValidationError); // Allow the device to be reused.

      throw new TestFailedButDeviceReusable(`Unexpected validation error occurred: ${gpuValidationError.message}`);
    }

    if (gpuOutOfMemoryError !== null) {
      assert(gpuOutOfMemoryError instanceof GPUOutOfMemoryError); // Don't allow the device to be reused; unexpected OOM could break the device.

      throw new TestOOMedShouldAttemptGC('Unexpected out-of-memory error occurred');
    }
  }

}
//# sourceMappingURL=device_pool.js.map