/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { Fixture } from '../common/framework/fixture.js';
import { DevicePool, TestOOMedShouldAttemptGC } from '../common/framework/gpu/device_pool.js';
import { attemptGarbageCollection } from '../common/framework/util/collect_garbage.js';
import { assert } from '../common/framework/util/util.js';
import { fillTextureDataWithTexelValue, getTextureCopyLayout } from './util/texture/layout.js';
import { getTexelDataRepresentation } from './util/texture/texelData.js';
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
  } // Note: finalize is called even if init was unsuccessful.


  async finalize() {
    await super.finalize();

    if (this.objects) {
      let threw;
      {
        const objects = this.objects;
        this.objects = undefined;

        try {
          await devicePool.release(objects.device);
        } catch (ex) {
          threw = ex;
        }
      } // The GPUDevice and GPUQueue should now have no outstanding references.

      if (threw) {
        if (threw instanceof TestOOMedShouldAttemptGC) {
          // Try to clean up, in case there are stray GPU resources in need of collection.
          await attemptGarbageCollection();
        }

        throw threw;
      }
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
    const dst = this.createCopyForMapRead(src, expected.buffer.byteLength);
    this.eventualAsyncExpectation(async niceStack => {
      const constructor = expected.constructor;
      const actual = new constructor((await dst.mapReadAsync()));
      const check = this.checkBuffer(actual, expected);

      if (check !== undefined) {
        niceStack.message = check;
        this.rec.expectationFailed(niceStack);
      }

      dst.destroy();
    });
  }

  expectBuffer(actual, exp) {
    const check = this.checkBuffer(actual, exp);

    if (check !== undefined) {
      this.rec.expectationFailed(new Error(check));
    }
  }

  checkBuffer(actual, exp, tolerance = 0) {
    assert(actual.constructor === exp.constructor);
    const size = exp.byteLength;

    if (actual.byteLength !== size) {
      return 'size mismatch';
    }

    const failedByteIndices = [];
    const failedByteExpectedValues = [];
    const failedByteActualValues = [];

    for (let i = 0; i < size; ++i) {
      const tol = typeof tolerance === 'function' ? tolerance(i) : tolerance;

      if (Math.abs(actual[i] - exp[i]) > tol) {
        if (failedByteIndices.length >= 4) {
          failedByteIndices.push('...');
          failedByteExpectedValues.push('...');
          failedByteActualValues.push('...');
          break;
        }

        failedByteIndices.push(i.toString());
        failedByteExpectedValues.push(exp[i].toString());
        failedByteActualValues.push(actual[i].toString());
      }
    }

    const summary = `at [${failedByteIndices.join(', ')}], \
expected [${failedByteExpectedValues.join(', ')}], \
got [${failedByteActualValues.join(', ')}]`;
    const lines = [summary]; // TODO: Could make a more convenient message, which could look like e.g.:
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

    if (size <= 256 && failedByteIndices.length > 0) {
      const expHex = Array.from(new Uint8Array(exp.buffer, exp.byteOffset, exp.byteLength)).map(x => x.toString(16).padStart(2, '0')).join('');
      const actHex = Array.from(new Uint8Array(actual.buffer, actual.byteOffset, actual.byteLength)).map(x => x.toString(16).padStart(2, '0')).join('');
      lines.push('EXPECT:\t  ' + exp.join(' '));
      lines.push('\t0x' + expHex);
      lines.push('ACTUAL:\t  ' + actual.join(' '));
      lines.push('\t0x' + actHex);
    }

    if (failedByteIndices.length) {
      return lines.join('\n');
    }

    return undefined;
  }

  expectSingleColor(src, format, {
    size,
    exp,
    dimension = '2d',
    slice = 0,
    layout
  }) {
    const {
      byteLength,
      bytesPerRow,
      rowsPerImage,
      mipSize
    } = getTextureCopyLayout(format, dimension, size, layout);
    const expectedTexelData = getTexelDataRepresentation(format).getBytes(exp);
    const buffer = this.device.createBuffer({
      size: byteLength,
      usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
    });
    const commandEncoder = this.device.createCommandEncoder();
    commandEncoder.copyTextureToBuffer({
      texture: src,
      mipLevel: layout?.mipLevel,
      arrayLayer: slice
    }, {
      buffer,
      bytesPerRow,
      rowsPerImage
    }, mipSize);
    this.queue.submit([commandEncoder.finish()]);
    const arrayBuffer = new ArrayBuffer(byteLength);
    fillTextureDataWithTexelValue(expectedTexelData, format, dimension, arrayBuffer, size, layout);
    this.expectContents(buffer, new Uint8Array(arrayBuffer));
  }

}
//# sourceMappingURL=gpu_test.js.map