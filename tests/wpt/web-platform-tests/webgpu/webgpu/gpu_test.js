/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { Fixture } from '../common/framework/fixture.js';
import { getGPU } from '../common/framework/gpu/implementation.js';
import { assert, unreachable } from '../common/framework/util/util.js';

class DevicePool {
  constructor() {
    _defineProperty(this, "device", undefined);

    _defineProperty(this, "state", 'uninitialized');
  }

  async initialize() {
    try {
      const gpu = getGPU();
      const adapter = await gpu.requestAdapter();
      this.device = await adapter.requestDevice();
    } catch (ex) {
      this.state = 'failed';
      throw ex;
    }
  }

  async acquire() {
    assert(this.state !== 'acquired', 'Device was in use');
    assert(this.state !== 'failed', 'Failed to initialize WebGPU device');
    const state = this.state;
    this.state = 'acquired';

    if (state === 'uninitialized') {
      await this.initialize();
    }

    assert(!!this.device);
    return this.device;
  }

  release(device) {
    assert(this.state === 'acquired');
    assert(device === this.device, 'Released device was the wrong device');
    this.state = 'free';
  }

}

const devicePool = new DevicePool();
export class GPUTest extends Fixture {
  constructor(...args) {
    super(...args);

    _defineProperty(this, "objects", undefined);

    _defineProperty(this, "initialized", false);
  }

  get device() {
    assert(this.objects !== undefined);
    return this.objects.device;
  }

  get queue() {
    assert(this.objects !== undefined);
    return this.objects.queue;
  }

  async init() {
    await super.init();
    const device = await devicePool.acquire();
    const queue = device.defaultQueue;
    this.objects = {
      device,
      queue
    };

    try {
      await device.popErrorScope();
      unreachable('There was an error scope on the stack at the beginning of the test');
    } catch (ex) {}

    device.pushErrorScope('out-of-memory');
    device.pushErrorScope('validation');
    this.initialized = true;
  }

  async finalize() {
    // Note: finalize is called even if init was unsuccessful.
    await super.finalize();

    if (this.initialized) {
      const gpuValidationError = await this.device.popErrorScope();

      if (gpuValidationError !== null) {
        assert(gpuValidationError instanceof GPUValidationError);
        this.fail(`Unexpected validation error occurred: ${gpuValidationError.message}`);
      }

      const gpuOutOfMemoryError = await this.device.popErrorScope();

      if (gpuOutOfMemoryError !== null) {
        assert(gpuOutOfMemoryError instanceof GPUOutOfMemoryError);
        this.fail('Unexpected out-of-memory error occurred');
      }
    }

    if (this.objects) {
      devicePool.release(this.objects.device);
    }
  }

  createCopyForMapRead(src, size) {
    const dst = this.device.createBuffer({
      size,
      usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST
    });
    const c = this.device.createCommandEncoder();
    c.copyBufferToBuffer(src, 0, dst, 0, size);
    this.queue.submit([c.finish()]);
    return dst;
  } // TODO: add an expectContents for textures, which logs data: uris on failure


  expectContents(src, expected) {
    const exp = new Uint8Array(expected.buffer, expected.byteOffset, expected.byteLength);
    const dst = this.createCopyForMapRead(src, expected.buffer.byteLength);
    this.eventualAsyncExpectation(async niceStack => {
      const actual = new Uint8Array((await dst.mapReadAsync()));
      const check = this.checkBuffer(actual, exp);

      if (check !== undefined) {
        niceStack.message = check;
        this.rec.fail(niceStack);
      }

      dst.destroy();
    });
  }

  expectBuffer(actual, exp) {
    const check = this.checkBuffer(actual, exp);

    if (check !== undefined) {
      this.rec.fail(new Error(check));
    }
  }

  checkBuffer(actual, exp) {
    const size = exp.byteLength;

    if (actual.byteLength !== size) {
      return 'size mismatch';
    }

    const lines = [];
    let failedPixels = 0;

    for (let i = 0; i < size; ++i) {
      if (actual[i] !== exp[i]) {
        if (failedPixels > 4) {
          lines.push('... and more');
          break;
        }

        failedPixels++;
        lines.push(`at [${i}], expected ${exp[i]}, got ${actual[i]}`);
      }
    } // TODO: Could make a more convenient message, which could look like e.g.:
    //
    //   Starting at offset 48,
    //              got 22222222 ABCDABCD 99999999
    //     but expected 22222222 55555555 99999999
    //
    // or
    //
    //   Starting at offset 0,
    //              got 00000000 00000000 00000000 00000000 (... more)
    //     but expected 00FF00FF 00FF00FF 00FF00FF 00FF00FF (... more)
    //
    // Or, maybe these diffs aren't actually very useful (given we have the prints just above here),
    // and we should remove them. More important will be logging of texture data in a visual format.


    if (size <= 256 && failedPixels > 0) {
      const expHex = Array.from(exp).map(x => x.toString(16).padStart(2, '0')).join('');
      const actHex = Array.from(actual).map(x => x.toString(16).padStart(2, '0')).join('');
      lines.push('EXPECT: ' + expHex);
      lines.push('ACTUAL: ' + actHex);
    }

    if (failedPixels) {
      return lines.join('\n');
    }

    return undefined;
  }

}
//# sourceMappingURL=gpu_test.js.map