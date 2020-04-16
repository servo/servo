/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

import { unreachable } from '../../../common/framework/util/util.js';
import { GPUTest } from '../../gpu_test.js';
export let BindingResourceType;

(function (BindingResourceType) {
  BindingResourceType["error-buffer"] = "error-buffer";
  BindingResourceType["error-sampler"] = "error-sampler";
  BindingResourceType["error-textureview"] = "error-textureview";
  BindingResourceType["uniform-buffer"] = "uniform-buffer";
  BindingResourceType["storage-buffer"] = "storage-buffer";
  BindingResourceType["sampler"] = "sampler";
  BindingResourceType["sampled-textureview"] = "sampled-textureview";
  BindingResourceType["storage-textureview"] = "storage-textureview";
})(BindingResourceType || (BindingResourceType = {}));

export function resourceBindingMatches(b, r) {
  switch (b) {
    case 'storage-buffer':
    case 'readonly-storage-buffer':
      return r === 'storage-buffer';

    case 'sampled-texture':
      return r === 'sampled-textureview';

    case 'sampler':
      return r === 'sampler';

    case 'readonly-storage-texture':
    case 'writeonly-storage-texture':
      return r === 'storage-textureview';

    case 'uniform-buffer':
      return r === 'uniform-buffer';

    default:
      unreachable('unknown GPUBindingType');
  }
}
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
      case 'error-buffer':
        return {
          buffer: this.getErrorBuffer()
        };

      case 'error-sampler':
        return this.getErrorSampler();

      case 'error-textureview':
        return this.getErrorTextureView();

      case 'uniform-buffer':
        return {
          buffer: this.getUniformBuffer()
        };

      case 'storage-buffer':
        return {
          buffer: this.getStorageBuffer()
        };

      case 'sampler':
        return this.getSampler();

      case 'sampled-textureview':
        return this.getSampledTexture().createView();

      case 'storage-textureview':
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
        this.rec.fail(niceStack);
      } else if (gpuValidationError instanceof GPUValidationError) {
        niceStack.message = `Captured validation error - ${gpuValidationError.message}`;
        this.rec.debug(niceStack);
      }
    });
  }

}
//# sourceMappingURL=validation_test.js.map