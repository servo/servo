/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { getGPU } from '../../framework/gpu/implementation.js';
import { Fixture } from '../../framework/index.js'; // TODO: Should this gain some functionality currently only in UnitTest?

export class GPUTest extends Fixture {
  constructor(...args) {
    super(...args);

    _defineProperty(this, "device", undefined);

    _defineProperty(this, "queue", undefined);
  }

  async init() {
    super.init();
    const gpu = getGPU();
    const adapter = await gpu.requestAdapter();
    this.device = await adapter.requestDevice({});
    this.queue = this.device.getQueue();
  } // TODO: add an expectContents for textures, which logs data: uris on failure


  expectContents(src, expected) {
    return this.asyncExpectation(async () => {
      const exp = new Uint8Array(expected.buffer, expected.byteOffset, expected.byteLength);
      const size = expected.buffer.byteLength;
      const dst = this.device.createBuffer({
        size: expected.buffer.byteLength,
        usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST
      });
      const c = this.device.createCommandEncoder({});
      c.copyBufferToBuffer(src, 0, dst, 0, size);
      this.queue.submit([c.finish()]);
      const actual = new Uint8Array((await dst.mapReadAsync()));
      this.expectBuffer(actual, exp);
    });
  }

  expectBuffer(actual, exp) {
    const size = exp.byteLength;

    if (actual.byteLength !== size) {
      this.rec.fail('size mismatch');
      return;
    }

    let failedPixels = 0;

    for (let i = 0; i < size; ++i) {
      if (actual[i] !== exp[i]) {
        if (failedPixels > 4) {
          this.rec.fail('... and more');
          break;
        }

        failedPixels++;
        this.rec.fail(`at [${i}], expected ${exp[i]}, got ${actual[i]}`);
      }
    }

    if (size <= 256 && failedPixels > 0) {
      const expHex = Array.from(exp).map(x => x.toString(16).padStart(2, '0')).join('');
      const actHex = Array.from(actual).map(x => x.toString(16).padStart(2, '0')).join('');
      this.rec.log('EXPECT: ' + expHex);
      this.rec.log('ACTUAL: ' + actHex);
    }
  }

}
//# sourceMappingURL=gpu_test.js.map