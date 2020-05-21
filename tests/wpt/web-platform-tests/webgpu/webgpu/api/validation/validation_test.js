/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

import { unreachable } from '../../../common/framework/util/util.js';
import { GPUTest } from '../../gpu_test.js';
export class ValidationTest extends GPUTest {
  getStorageBuffer() {
    return this.device.createBuffer({
      size: 1024,
      usage: GPUBufferUsage.STORAGE
    });
  }

  getUniformBuffer() {
    return this.device.createBuffer({
      size: 1024,
      usage: GPUBufferUsage.UNIFORM
    });
  }

  getErrorBuffer() {
    this.device.pushErrorScope('validation');
    const errorBuffer = this.device.createBuffer({
      size: 1024,
      usage: 0xffff // Invalid GPUBufferUsage

    });
    this.device.popErrorScope();
    return errorBuffer;
  }

  getSampler() {
    return this.device.createSampler();
  }

  getComparisonSampler() {
    return this.device.createSampler({
      compare: 'never'
    });
  }

  getErrorSampler() {
    this.device.pushErrorScope('validation');
    const sampler = this.device.createSampler({
      lodMinClamp: -1
    });
    this.device.popErrorScope();
    return sampler;
  }

  getSampledTexture() {
    return this.device.createTexture({
      size: {
        width: 16,
        height: 16,
        depth: 1
      },
      format: 'rgba8unorm',
      usage: GPUTextureUsage.SAMPLED
    });
  }

  getStorageTexture() {
    return this.device.createTexture({
      size: {
        width: 16,
        height: 16,
        depth: 1
      },
      format: 'rgba8unorm',
      usage: GPUTextureUsage.STORAGE
    });
  }

  getErrorTextureView() {
    this.device.pushErrorScope('validation');
    const view = this.device.createTexture({
      size: {
        width: 0,
        height: 0,
        depth: 0
      },
      format: 'rgba8unorm',
      usage: GPUTextureUsage.SAMPLED
    }).createView();
    this.device.popErrorScope();
    return view;
  }

  getBindingResource(bindingType) {
    switch (bindingType) {
      case 'errorBuf':
        return {
          buffer: this.getErrorBuffer()
        };

      case 'errorSamp':
        return this.getErrorSampler();

      case 'errorTex':
        return this.getErrorTextureView();

      case 'uniformBuf':
        return {
          buffer: this.getUniformBuffer()
        };

      case 'storageBuf':
        return {
          buffer: this.getStorageBuffer()
        };

      case 'plainSamp':
        return this.getSampler();

      case 'compareSamp':
        return this.getComparisonSampler();

      case 'sampledTex':
        return this.getSampledTexture().createView();

      case 'storageTex':
        return this.getStorageTexture().createView();

      default:
        unreachable('unknown binding resource type');
    }
  }

  expectValidationError(fn, shouldError = true) {
    // If no error is expected, we let the scope surrounding the test catch it.
    if (shouldError === false) {
      fn();
      return;
    }

    this.device.pushErrorScope('validation');
    fn();
    const promise = this.device.popErrorScope();
    this.eventualAsyncExpectation(async niceStack => {
      const gpuValidationError = await promise;

      if (!gpuValidationError) {
        niceStack.message = 'Validation error was expected.';
        this.rec.validationFailed(niceStack);
      } else if (gpuValidationError instanceof GPUValidationError) {
        niceStack.message = `Captured validation error - ${gpuValidationError.message}`;
        this.rec.debug(niceStack);
      }
    });
  }

}
//# sourceMappingURL=validation_test.js.map