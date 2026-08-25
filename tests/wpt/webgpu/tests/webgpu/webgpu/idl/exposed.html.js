/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/ // WPT-specific test checking that WebGPU is available iff isSecureContext.
import { assert } from '../../common/util/util.js';
const items = [
globalThis.navigator.gpu,
globalThis.GPU,
globalThis.GPUAdapter,
globalThis.GPUAdapterInfo,
globalThis.GPUBindGroup,
globalThis.GPUBindGroupLayout,
globalThis.GPUBuffer,
globalThis.GPUBufferUsage,
globalThis.GPUCanvasContext,
globalThis.GPUColorWrite,
globalThis.GPUCommandBuffer,
globalThis.GPUCommandEncoder,
globalThis.GPUCompilationInfo,
globalThis.GPUCompilationMessage,
globalThis.GPUComputePassEncoder,
globalThis.GPUComputePipeline,
globalThis.GPUDevice,
globalThis.GPUDeviceLostInfo,
globalThis.GPUError,
globalThis.GPUExternalTexture,
globalThis.GPUMapMode,
globalThis.GPUOutOfMemoryError,
globalThis.GPUPipelineLayout,
globalThis.GPUQuerySet,
globalThis.GPUQueue,
globalThis.GPURenderBundle,
globalThis.GPURenderBundleEncoder,
globalThis.GPURenderPassEncoder,
globalThis.GPURenderPipeline,
globalThis.GPUSampler,
globalThis.GPUShaderModule,
globalThis.GPUShaderStage,
globalThis.GPUSupportedLimits,
globalThis.GPUTexture,
globalThis.GPUTextureUsage,
globalThis.GPUTextureView,
globalThis.GPUUncapturedErrorEvent,
globalThis.GPUValidationError];


for (const item of items) {
  if (globalThis.isSecureContext) {
    assert(item !== undefined, 'Item/interface should be exposed on secure context');
  } else {
    assert(item === undefined, 'Item/interface should not be exposed on insecure context');
  }
}