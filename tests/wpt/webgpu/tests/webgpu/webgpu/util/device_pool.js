/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { SkipTestCase } from '../../common/framework/fixture.js';
import { attemptGarbageCollection } from '../../common/util/collect_garbage.js';
import { getGPU } from '../../common/util/navigator_gpu.js';
import {
  assert,
  raceWithRejectOnTimeout,
  assertReject,
  unreachable,
} from '../../common/util/util.js';
import { kLimitInfo, kLimits } from '../capability_info.js';

class TestFailedButDeviceReusable extends Error {}
class FeaturesNotSupported extends Error {}
export class TestOOMedShouldAttemptGC extends Error {}

export class DevicePool {
  holders = 'uninitialized';

  /** Acquire a device from the pool and begin the error scopes. */
  async acquire(recorder, descriptor) {
    let errorMessage = '';
    if (this.holders === 'uninitialized') {
      this.holders = new DescriptorToHolderMap();
      try {
        await this.holders.getOrCreate(recorder, undefined);
      } catch (ex) {
        this.holders = 'failed';
        if (ex instanceof Error) {
          errorMessage = ` with ${ex.name} "${ex.message}"`;
        }
      }
    }

    assert(
      this.holders !== 'failed',
      `WebGPU device failed to initialize${errorMessage}; not retrying`
    );

    const holder = await this.holders.getOrCreate(recorder, descriptor);

    assert(holder.state === 'free', 'Device was in use on DevicePool.acquire');
    holder.state = 'acquired';
    holder.beginTestScope();
    return holder;
  }

  /**
   * End the error scopes and check for errors.
   * Then, if the device seems reusable, release it back into the pool. Otherwise, drop it.
   */
  async release(holder) {
    assert(this.holders instanceof DescriptorToHolderMap, 'DevicePool got into a bad state');
    assert(holder instanceof DeviceHolder, 'DeviceProvider should always be a DeviceHolder');

    assert(holder.state === 'acquired', 'trying to release a device while already released');
    try {
      await holder.endTestScope();

      // (Hopefully if the device was lost, it has been reported by the time endErrorScopes()
      // has finished (or timed out). If not, it could cause a finite number of extra test
      // failures following this one (but should recover eventually).)
      assert(
        holder.lostInfo === undefined,
        `Device was unexpectedly lost. Reason: ${holder.lostInfo?.reason}, Message: ${holder.lostInfo?.message}`
      );
    } catch (ex) {
      // Any error that isn't explicitly TestFailedButDeviceReusable forces a new device to be
      // created for the next test.
      if (!(ex instanceof TestFailedButDeviceReusable)) {
        this.holders.delete(holder);
        if ('destroy' in holder.device) {
          holder.device.destroy();
        }

        // Release the (hopefully only) ref to the GPUDevice.
        holder.releaseGPUDevice();

        // Try to clean up, in case there are stray GPU resources in need of collection.
        if (ex instanceof TestOOMedShouldAttemptGC) {
          await attemptGarbageCollection();
        }
      }
      // In the try block, we may throw an error if the device is lost in order to force device
      // reinitialization, however, if the device lost was expected we want to suppress the error
      // The device lost is expected when `holder.expectedLostReason` is equal to
      // `holder.lostInfo.reason`.
      const expectedDeviceLost =
        holder.expectedLostReason !== undefined &&
        holder.lostInfo !== undefined &&
        holder.expectedLostReason === holder.lostInfo.reason;
      if (!expectedDeviceLost) {
        throw ex;
      }
    } finally {
      // Mark the holder as free so the device can be reused (if it's still in this.devices).
      holder.state = 'free';
    }
  }
}

/**
 * Map from GPUDeviceDescriptor to DeviceHolder.
 */
class DescriptorToHolderMap {
  /** Map keys that are known to be unsupported and can be rejected quickly. */
  unsupported = new Set();
  holders = new Map();

  /** Deletes an item from the map by DeviceHolder value. */
  delete(holder) {
    for (const [k, v] of this.holders) {
      if (v === holder) {
        this.holders.delete(k);
        return;
      }
    }
    unreachable("internal error: couldn't find DeviceHolder to delete");
  }

  /**
   * Gets a DeviceHolder from the map if it exists; otherwise, calls create() to create one,
   * inserts it, and returns it.
   *
   * If an `uncanonicalizedDescriptor` is provided, it is canonicalized and used as the map key.
   * If one is not provided, the map key is `""` (empty string).
   *
   * Throws SkipTestCase if devices with this descriptor are unsupported.
   */
  async getOrCreate(recorder, uncanonicalizedDescriptor) {
    const [descriptor, key] = canonicalizeDescriptor(uncanonicalizedDescriptor);
    // Quick-reject descriptors that are known to be unsupported already.
    if (this.unsupported.has(key)) {
      throw new SkipTestCase(
        `GPUDeviceDescriptor previously failed: ${JSON.stringify(descriptor)}`
      );
    }

    // Search for an existing device with the same descriptor.
    {
      const value = this.holders.get(key);
      if (value) {
        // Move it to the end of the Map (most-recently-used).
        this.holders.delete(key);
        this.holders.set(key, value);
        return value;
      }
    }

    // No existing item was found; add a new one.
    let value;
    try {
      value = await DeviceHolder.create(recorder, descriptor);
    } catch (ex) {
      if (ex instanceof FeaturesNotSupported) {
        this.unsupported.add(key);
        throw new SkipTestCase(
          `GPUDeviceDescriptor not supported: ${JSON.stringify(descriptor)}\n${ex?.message ?? ''}`
        );
      }

      throw ex;
    }
    this.insertAndCleanUp(key, value);
    return value;
  }

  /** Insert an entry, then remove the least-recently-used items if there are too many. */
  insertAndCleanUp(key, value) {
    this.holders.set(key, value);

    const kMaxEntries = 5;
    if (this.holders.size > kMaxEntries) {
      // Delete the first (least recently used) item in the set.
      for (const [key] of this.holders) {
        this.holders.delete(key);
        return;
      }
    }
  }
}

/**
 * Make a stringified map-key from a GPUDeviceDescriptor.
 * Tries to make sure all defaults are resolved, first - but it's okay if some are missed
 * (it just means some GPUDevice objects won't get deduplicated).
 *
 * This does **not** canonicalize `undefined` (the "default" descriptor) into a fully-qualified
 * GPUDeviceDescriptor. This is just because `undefined` is a common case and we want to use it
 * as a sanity check that WebGPU is working.
 */
function canonicalizeDescriptor(desc) {
  if (desc === undefined) {
    return [undefined, ''];
  }

  const featuresCanonicalized = desc.requiredFeatures
    ? Array.from(new Set(desc.requiredFeatures)).sort()
    : [];

  /** Canonicalized version of the requested limits: in canonical order, with only values which are
   * specified _and_ non-default. */
  const limitsCanonicalized = {};
  if (desc.requiredLimits) {
    for (const limit of kLimits) {
      const requestedValue = desc.requiredLimits[limit];
      const defaultValue = kLimitInfo[limit].default;
      // Skip adding a limit to limitsCanonicalized if it is the same as the default.
      if (requestedValue !== undefined && requestedValue !== defaultValue) {
        limitsCanonicalized[limit] = requestedValue;
      }
    }
  }

  // Type ensures every field is carried through.
  const descriptorCanonicalized = {
    requiredFeatures: featuresCanonicalized,
    requiredLimits: limitsCanonicalized,
    defaultQueue: {},
  };
  return [descriptorCanonicalized, JSON.stringify(descriptorCanonicalized)];
}

function supportsFeature(adapter, descriptor) {
  if (descriptor === undefined) {
    return true;
  }

  for (const feature of descriptor.requiredFeatures) {
    if (!adapter.features.has(feature)) {
      return false;
    }
  }

  return true;
}

/**
 * DeviceHolder has three states:
 * - 'free': Free to be used for a new test.
 * - 'acquired': In use by a running test.
 */

/**
 * Holds a GPUDevice and tracks its state (free/acquired) and handles device loss.
 */
class DeviceHolder {
  /** The device. Will be cleared during cleanup if there were unexpected errors. */

  /** Whether the device is in use by a test or not. */
  state = 'free';
  /** initially undefined; becomes set when the device is lost */

  // Gets a device and creates a DeviceHolder.
  // If the device is lost, DeviceHolder.lost gets set.
  static async create(recorder, descriptor) {
    const gpu = getGPU(recorder);
    const adapter = await gpu.requestAdapter();
    assert(adapter !== null, 'requestAdapter returned null');
    if (!supportsFeature(adapter, descriptor)) {
      throw new FeaturesNotSupported('One or more features are not supported');
    }
    const device = await adapter.requestDevice(descriptor);
    assert(device !== null, 'requestDevice returned null');

    return new DeviceHolder(device);
  }

  constructor(device) {
    this._device = device;
    void this._device.lost.then(ev => {
      this.lostInfo = ev;
    });
  }

  get device() {
    assert(this._device !== undefined);
    return this._device;
  }

  /** Push error scopes that surround test execution. */
  beginTestScope() {
    assert(this.state === 'acquired');
    this.device.pushErrorScope('validation');
    this.device.pushErrorScope('internal');
    this.device.pushErrorScope('out-of-memory');
  }

  /** Mark the DeviceHolder as expecting a device loss when the test scope ends. */
  expectDeviceLost(reason) {
    assert(this.state === 'acquired');
    this.expectedLostReason = reason;
  }

  /**
   * Attempt to end test scopes: Check that there are no extra error scopes, and that no
   * otherwise-uncaptured errors occurred during the test. Time out if it takes too long.
   */
  endTestScope() {
    assert(this.state === 'acquired');
    const kTimeout = 5000;

    // Time out if attemptEndTestScope (popErrorScope or onSubmittedWorkDone) never completes. If
    // this rejects, the device won't be reused, so it's OK that popErrorScope calls may not have
    // finished.
    //
    // This could happen due to a browser bug - e.g.,
    // as of this writing, on Chrome GPU process crash, popErrorScope just hangs.
    return raceWithRejectOnTimeout(this.attemptEndTestScope(), kTimeout, 'endTestScope timed out');
  }

  async attemptEndTestScope() {
    let gpuValidationError;
    let gpuInternalError;
    let gpuOutOfMemoryError;

    // Submit to the queue to attempt to force a GPU flush.
    this.device.queue.submit([]);

    try {
      // May reject if the device was lost.
      [gpuOutOfMemoryError, gpuInternalError, gpuValidationError] = await Promise.all([
        this.device.popErrorScope(),
        this.device.popErrorScope(),
        this.device.popErrorScope(),
      ]);
    } catch (ex) {
      assert(this.lostInfo !== undefined, 'popErrorScope failed; did beginTestScope get missed?');
      throw ex;
    }

    // Attempt to wait for the queue to be idle.
    if (this.device.queue.onSubmittedWorkDone) {
      await this.device.queue.onSubmittedWorkDone();
    }

    await assertReject(
      this.device.popErrorScope(),
      'There was an extra error scope on the stack after a test'
    );

    if (gpuOutOfMemoryError !== null) {
      assert(gpuOutOfMemoryError instanceof GPUOutOfMemoryError);
      // Don't allow the device to be reused; unexpected OOM could break the device.
      throw new TestOOMedShouldAttemptGC('Unexpected out-of-memory error occurred');
    }
    if (gpuInternalError !== null) {
      assert(gpuInternalError instanceof GPUInternalError);
      // Allow the device to be reused.
      throw new TestFailedButDeviceReusable(
        `Unexpected internal error occurred: ${gpuInternalError.message}`
      );
    }
    if (gpuValidationError !== null) {
      assert(gpuValidationError instanceof GPUValidationError);
      // Allow the device to be reused.
      throw new TestFailedButDeviceReusable(
        `Unexpected validation error occurred: ${gpuValidationError.message}`
      );
    }
  }

  /**
   * Release the ref to the GPUDevice. This should be the only ref held by the DevicePool or
   * GPUTest, so in theory it can get garbage collected.
   */
  releaseGPUDevice() {
    this._device = undefined;
  }
}
