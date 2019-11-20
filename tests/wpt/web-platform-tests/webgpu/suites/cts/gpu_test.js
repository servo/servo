/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { getGPU } from '../../framework/gpu/implementation.js';
import { Fixture } from '../../framework/index.js';
let glslangInstance;
export class GPUTest extends Fixture {
  constructor(...args) {
    super(...args);

    _defineProperty(this, "device", undefined);

    _defineProperty(this, "queue", undefined);

    _defineProperty(this, "initialized", false);

    _defineProperty(this, "supportsSPIRV", true);
  }

  async init() {
    super.init();
    const gpu = getGPU();
    const adapter = await gpu.requestAdapter();
    this.device = await adapter.requestDevice();
    this.queue = this.device.defaultQueue;
    const isSafari = /^((?!chrome|android).)*safari/i.test(navigator.userAgent);

    if (isSafari) {
      this.supportsSPIRV = false;
    }

    try {
      await this.device.popErrorScope();
      throw new Error('There was an error scope on the stack at the beginning of the test');
    } catch (ex) {}

    this.device.pushErrorScope('out-of-memory');
    this.device.pushErrorScope('validation');
    this.initialized = true;
  }

  async finalize() {
    super.finalize();

    if (this.initialized) {
      const gpuValidationError = await this.device.popErrorScope();

      if (gpuValidationError !== null) {
        if (!(gpuValidationError instanceof GPUValidationError)) throw new Error();
        this.fail(`Unexpected validation error occurred: ${gpuValidationError.message}`);
      }

      const gpuOutOfMemoryError = await this.device.popErrorScope();

      if (gpuOutOfMemoryError !== null) {
        if (!(gpuOutOfMemoryError instanceof GPUOutOfMemoryError)) throw new Error();
        this.fail('Unexpected out-of-memory error occurred');
      }
    }
  }

  async initGLSL() {
    if (!glslangInstance) {
      const glslangPath = '../../glslang.js';
      let glslangModule;

      try {
        glslangModule = (await import(glslangPath)).default;
      } catch (ex) {
        this.skip('glslang is not available');
      }

      await new Promise(resolve => {
        glslangModule().then(glslang => {
          glslangInstance = glslang;
          resolve();
        });
      });
    }
  }

  createShaderModule(desc) {
    if (!this.supportsSPIRV) {
      this.skip('SPIR-V not available');
    }

    return this.device.createShaderModule(desc);
  }

  makeShaderModuleFromGLSL(stage, glsl) {
    if (!glslangInstance) {
      throw new Error('GLSL compiler is not instantiated. Run `await t.initGLSL()` first');
    }

    const code = glslangInstance.compileGLSL(glsl, stage, false);
    return this.device.createShaderModule({
      code
    });
  } // TODO: add an expectContents for textures, which logs data: uris on failure


  expectContents(src, expected) {
    const exp = new Uint8Array(expected.buffer, expected.byteOffset, expected.byteLength);
    const size = expected.buffer.byteLength;
    const dst = this.device.createBuffer({
      size: expected.buffer.byteLength,
      usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST
    });
    const c = this.device.createCommandEncoder();
    c.copyBufferToBuffer(src, 0, dst, 0, size);
    this.queue.submit([c.finish()]);
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
    }

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