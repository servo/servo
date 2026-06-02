/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import {
  kMaxQueryCount } from

'../../capability_info.js';


export function createTextureWithState(
t,
state,
descriptor)
{
  descriptor = descriptor ?? {
    size: { width: 1, height: 1, depthOrArrayLayers: 1 },
    format: 'rgba8unorm',
    usage:
    GPUTextureUsage.COPY_SRC |
    GPUTextureUsage.COPY_DST |
    GPUTextureUsage.TEXTURE_BINDING |
    GPUTextureUsage.STORAGE_BINDING |
    GPUTextureUsage.RENDER_ATTACHMENT
  };

  switch (state) {
    case 'valid':
      return t.createTextureTracked(descriptor);
    case 'invalid':
      return getErrorTexture(t);
    case 'destroyed':{
        const texture = t.createTextureTracked(descriptor);
        texture.destroy();
        return texture;
      }
  }
}

/**
 * Create a GPUTexture in the specified state. A `descriptor` may optionally be passed;
 * if `state` is `'invalid'`, it will be modified to add an invalid combination of usages.
 */
export function createBufferWithState(
t,
state,
descriptor)
{
  descriptor = descriptor ?? {
    size: 4,
    usage: GPUBufferUsage.VERTEX
  };

  switch (state) {
    case 'valid':
      return t.createBufferTracked(descriptor);

    case 'invalid':{
        // Make the buffer invalid because of an invalid combination of usages but keep the
        // descriptor passed as much as possible (for mappedAtCreation and friends).
        t.device.pushErrorScope('validation');
        const buffer = t.createBufferTracked({
          ...descriptor,
          usage: descriptor.usage | GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_SRC
        });
        void t.device.popErrorScope();
        return buffer;
      }
    case 'destroyed':{
        const buffer = t.createBufferTracked(descriptor);
        buffer.destroy();
        return buffer;
      }
  }
}

/**
 * Create a GPUQuerySet in the specified state.
 * A `descriptor` may optionally be passed, which is used when `state` is not `'invalid'`.
 */
export function createQuerySetWithState(
t,
state,
desc)
{
  const descriptor = { type: 'occlusion', count: 2, ...desc };

  switch (state) {
    case 'valid':
      return t.createQuerySetTracked(descriptor);
    case 'invalid':{
        // Make the queryset invalid because of the count out of bounds.
        descriptor.count = kMaxQueryCount + 1;
        return t.expectGPUError('validation', () => t.createQuerySetTracked(descriptor));
      }
    case 'destroyed':{
        const queryset = t.createQuerySetTracked(descriptor);
        queryset.destroy();
        return queryset;
      }
  }
}

/** Create an arbitrarily-sized GPUBuffer with the STORAGE usage. */
export function getStorageBuffer(t) {
  return t.createBufferTracked({ size: 1024, usage: GPUBufferUsage.STORAGE });
}

/** Create an arbitrarily-sized GPUBuffer with the UNIFORM usage. */
export function getUniformBuffer(t) {
  return t.createBufferTracked({ size: 1024, usage: GPUBufferUsage.UNIFORM });
}

/** Return an invalid GPUBuffer. */
export function getErrorBuffer(t) {
  return createBufferWithState(t, 'invalid');
}

/** Return an invalid GPUSampler. */
export function getErrorSampler(t) {
  t.device.pushErrorScope('validation');
  const sampler = t.device.createSampler({ lodMinClamp: -1 });
  void t.device.popErrorScope();
  return sampler;
}

/**
 * Return an arbitrarily-configured GPUTexture with the `TEXTURE_BINDING` usage and specified
 * sampleCount. The `RENDER_ATTACHMENT` usage will also be specified if sampleCount > 1 as is
 * required by WebGPU SPEC.
 */
export function getSampledTexture(t, sampleCount = 1) {
  const usage =
  sampleCount > 1 ?
  GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.RENDER_ATTACHMENT :
  GPUTextureUsage.TEXTURE_BINDING;
  return t.createTextureTracked({
    size: { width: 16, height: 16, depthOrArrayLayers: 1 },
    format: 'rgba8unorm',
    usage,
    sampleCount
  });
}

/** Return an arbitrarily-configured GPUTexture with the `STORAGE_BINDING` usage. */
function getStorageTexture(t, format) {
  return t.createTextureTracked({
    size: { width: 16, height: 16, depthOrArrayLayers: 1 },
    format,
    usage: GPUTextureUsage.STORAGE_BINDING
  });
}

/** Return an arbitrarily-configured GPUTexture with the `RENDER_ATTACHMENT` usage. */
export function getRenderTexture(t, sampleCount = 1) {
  return t.createTextureTracked({
    size: { width: 16, height: 16, depthOrArrayLayers: 1 },
    format: 'rgba8unorm',
    usage: GPUTextureUsage.RENDER_ATTACHMENT,
    sampleCount
  });
}

/** Return an invalid GPUTexture. */
export function getErrorTexture(t) {
  t.device.pushErrorScope('validation');
  const texture = t.createTextureTracked({
    size: { width: 0, height: 0, depthOrArrayLayers: 0 },
    format: 'rgba8unorm',
    usage: GPUTextureUsage.TEXTURE_BINDING
  });
  void t.device.popErrorScope();
  return texture;
}

/** Return an invalid GPUTextureView (created from an invalid GPUTexture). */
export function getErrorTextureView(t) {
  t.device.pushErrorScope('validation');
  const view = getErrorTexture(t).createView();
  void t.device.popErrorScope();
  return view;
}

/**
 * Return an arbitrary object of the specified {@link webgpu/capability_info!BindableResource} type
 * (e.g. `'errorBuf'`, `'nonFiltSamp'`, `sampledTexMS`, etc.)
 */
export function getBindingResource(t, bindingType) {
  switch (bindingType) {
    case 'errorBuf':
      return { buffer: getErrorBuffer(t) };
    case 'errorSamp':
      return getErrorSampler(t);
    case 'errorTex':
      return getErrorTextureView(t);
    case 'uniformBuf':
      return { buffer: getUniformBuffer(t) };
    case 'storageBuf':
      return { buffer: getStorageBuffer(t) };
    case 'filtSamp':
      return t.device.createSampler({ minFilter: 'linear' });
    case 'nonFiltSamp':
      return t.device.createSampler();
    case 'compareSamp':
      return t.device.createSampler({ compare: 'never' });
    case 'sampledTex':
      return getSampledTexture(t, 1).createView();
    case 'sampledTexMS':
      return getSampledTexture(t, 4).createView();
    case 'readonlyStorageTex':
    case 'writeonlyStorageTex':
    case 'readwriteStorageTex':
      return getStorageTexture(t, 'r32float').createView();
  }
}

/** Create an arbitrarily-sized GPUBuffer with the STORAGE usage from mismatched device. */
export function getDeviceMismatchedStorageBuffer(t) {
  return t.trackForCleanup(
    t.mismatchedDevice.createBuffer({ size: 4, usage: GPUBufferUsage.STORAGE })
  );
}

/** Create an arbitrarily-sized GPUBuffer with the UNIFORM usage from mismatched device. */
export function getDeviceMismatchedUniformBuffer(t) {
  return t.trackForCleanup(
    t.mismatchedDevice.createBuffer({ size: 4, usage: GPUBufferUsage.UNIFORM })
  );
}

/** Return a GPUTexture with descriptor from mismatched device. */
export function getDeviceMismatchedTexture(
t,
descriptor)
{
  return t.trackForCleanup(t.mismatchedDevice.createTexture(descriptor));
}

/** Return an arbitrarily-configured GPUTexture with the `SAMPLED` usage from mismatched device. */
export function getDeviceMismatchedSampledTexture(t, sampleCount = 1) {
  return getDeviceMismatchedTexture(t, {
    size: { width: 4, height: 4, depthOrArrayLayers: 1 },
    format: 'rgba8unorm',
    usage: GPUTextureUsage.TEXTURE_BINDING,
    sampleCount
  });
}

/** Return an arbitrarily-configured GPUTexture with the `STORAGE` usage from mismatched device. */
export function getDeviceMismatchedStorageTexture(
t,
format)
{
  return getDeviceMismatchedTexture(t, {
    size: { width: 4, height: 4, depthOrArrayLayers: 1 },
    format,
    usage: GPUTextureUsage.STORAGE_BINDING
  });
}

/** Return an arbitrarily-configured GPUTexture with the `RENDER_ATTACHMENT` usage from mismatched device. */
export function getDeviceMismatchedRenderTexture(t, sampleCount = 1) {
  return getDeviceMismatchedTexture(t, {
    size: { width: 4, height: 4, depthOrArrayLayers: 1 },
    format: 'rgba8unorm',
    usage: GPUTextureUsage.RENDER_ATTACHMENT,
    sampleCount
  });
}

export function getDeviceMismatchedBindingResource(
t,
bindingType)
{
  switch (bindingType) {
    case 'uniformBuf':
      return { buffer: getDeviceMismatchedUniformBuffer(t) };
    case 'storageBuf':
      return { buffer: getDeviceMismatchedStorageBuffer(t) };
    case 'filtSamp':
      return t.mismatchedDevice.createSampler({ minFilter: 'linear' });
    case 'nonFiltSamp':
      return t.mismatchedDevice.createSampler();
    case 'compareSamp':
      return t.mismatchedDevice.createSampler({ compare: 'never' });
    case 'sampledTex':
      return getDeviceMismatchedSampledTexture(t, 1).createView();
    case 'sampledTexMS':
      return getDeviceMismatchedSampledTexture(t, 4).createView();
    case 'readonlyStorageTex':
    case 'writeonlyStorageTex':
    case 'readwriteStorageTex':
      return getDeviceMismatchedStorageTexture(t, 'r32float').createView();
  }
}

/** Return a no-op shader code snippet for the specified shader stage. */
export function getNoOpShaderCode(stage) {
  switch (stage) {
    case 'VERTEX':
      return `
        @vertex fn main() -> @builtin(position) vec4<f32> {
          return vec4<f32>();
        }
      `;
    case 'FRAGMENT':
      return `@fragment fn main() {}`;
    case 'COMPUTE':
      return `@compute @workgroup_size(1) fn main() {}`;
  }
}

/** Create a GPURenderPipeline in the specified state. */
export function createRenderPipelineWithState(
t,
state)
{
  return state === 'valid' ? createNoOpRenderPipeline(t) : createErrorRenderPipeline(t);
}

/** Return a GPURenderPipeline with default options and no-op vertex and fragment shaders. */
export function createNoOpRenderPipeline(
t,
layout = 'auto',
colorFormat = 'rgba8unorm')
{
  return t.device.createRenderPipeline({
    layout,
    vertex: {
      module: t.device.createShaderModule({
        code: getNoOpShaderCode('VERTEX')
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: t.device.createShaderModule({
        code: getNoOpShaderCode('FRAGMENT')
      }),
      entryPoint: 'main',
      targets: [{ format: colorFormat, writeMask: 0 }]
    },
    primitive: { topology: 'triangle-list' }
  });
}

/** Return an invalid GPURenderPipeline. */
export function createErrorRenderPipeline(t) {
  t.device.pushErrorScope('validation');
  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code: ''
      }),
      entryPoint: ''
    }
  });
  void t.device.popErrorScope();
  return pipeline;
}

/** Return a GPUComputePipeline with a no-op shader. */
export function createNoOpComputePipeline(
t,
layout = 'auto')
{
  return t.device.createComputePipeline({
    layout,
    compute: {
      module: t.device.createShaderModule({
        code: getNoOpShaderCode('COMPUTE')
      }),
      entryPoint: 'main'
    }
  });
}

/** Return an invalid GPUComputePipeline. */
export function createErrorComputePipeline(t) {
  t.device.pushErrorScope('validation');
  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: ''
      }),
      entryPoint: ''
    }
  });
  void t.device.popErrorScope();
  return pipeline;
}

/** Return an invalid GPUShaderModule. */
export function createInvalidShaderModule(t) {
  t.device.pushErrorScope('validation');
  const code = 'deadbeaf'; // Something make no sense
  const shaderModule = t.device.createShaderModule({ code });
  void t.device.popErrorScope();
  return shaderModule;
}

/** Helper for testing createRenderPipeline(Async) validation */
export function doCreateRenderPipelineTest(
t,
isAsync,
_success,
descriptor,
errorTypeName = 'GPUPipelineError')
{
  if (isAsync) {
    if (_success) {
      t.shouldResolve(t.device.createRenderPipelineAsync(descriptor));
    } else {
      t.shouldReject(errorTypeName, t.device.createRenderPipelineAsync(descriptor));
    }
  } else {
    if (errorTypeName === 'GPUPipelineError') {
      t.expectValidationError(() => {
        t.device.createRenderPipeline(descriptor);
      }, !_success);
    } else {
      t.shouldThrow(_success ? false : errorTypeName, () => {
        t.device.createRenderPipeline(descriptor);
      });
    }
  }
}

/** Helper for testing createComputePipeline(Async) validation */
export function doCreateComputePipelineTest(
t,
isAsync,
_success,
descriptor,
errorTypeName = 'GPUPipelineError')
{
  if (isAsync) {
    if (_success) {
      t.shouldResolve(t.device.createComputePipelineAsync(descriptor));
    } else {
      t.shouldReject(errorTypeName, t.device.createComputePipelineAsync(descriptor));
    }
  } else {
    if (errorTypeName === 'GPUPipelineError') {
      t.expectValidationError(() => {
        t.device.createComputePipeline(descriptor);
      }, !_success);
    } else {
      t.shouldThrow(_success ? false : errorTypeName, () => {
        t.device.createComputePipeline(descriptor);
      });
    }
  }
}