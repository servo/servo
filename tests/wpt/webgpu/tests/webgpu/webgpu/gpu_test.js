/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { Fixture,


  SubcaseBatchState } from


'../common/framework/fixture.js';
import { registerShutdownTask } from '../common/framework/on_shutdown.js';
import { globalTestConfig, isCompatibilityDevice } from '../common/framework/test_config.js';
import { getGPU } from '../common/util/navigator_gpu.js';
import {
  assert,
  makeValueTestVariant,
  memcpy,
  range,



  unreachable } from
'../common/util/util.js';

import { kQueryTypeInfo } from './capability_info.js';

import {
  resolvePerAspectFormat,


  isCompressedTextureFormat,
  getRequiredFeatureForTextureFormat,
  isTextureFormatUsableAsRenderAttachment,
  isTextureFormatMultisampled,
  is32Float,
  isSintOrUintFormat,
  isTextureFormatResolvable,
  isDepthTextureFormat,
  isStencilTextureFormat,
  textureViewDimensionAndFormatCompatibleForDevice,
  textureDimensionAndFormatCompatibleForDevice,
  isTextureFormatUsableWithStorageAccessMode,
  isTextureFormatUsableWithCopyExternalImageToTexture } from
'./format_info.js';
import { checkElementsEqual, checkElementsBetween } from './util/check_contents.js';
import { CommandBufferMaker } from './util/command_buffer_maker.js';

import {


  DevicePool } from


'./util/device_pool.js';
import { align, roundDown } from './util/math.js';
import {
  getTextureCopyLayout,
  getTextureSubCopyLayout } from

'./util/texture/layout.js';
import { kTexelRepresentationInfo } from './util/texture/texel_data.js';
import { reifyExtent3D, reifyOrigin3D } from './util/unions.js';

// Declarations for WebGPU items we want tests for that are not yet officially part of the spec.










const devicePool = new DevicePool();

// MAINTENANCE_TODO: When DevicePool becomes able to provide multiple devices at once, use the
// usual one instead of a new one.
const mismatchedDevicePool = new DevicePool();

// On shutdown, try to explicitly destroy() the device pools (and devices) used by GPUTest,
// so they don't keep using system resources until they're fully garbage collected.
registerShutdownTask(() => {
  devicePool.destroy();
  mismatchedDevicePool.destroy();
});

const kResourceStateValues = ['valid', 'invalid', 'destroyed'];

export const kResourceStates = kResourceStateValues;

/** Various "convenient" shorthands for GPUDeviceDescriptors for selectDevice functions. */






export function initUncanonicalizedDeviceDescriptor(
descriptor)
{
  if (typeof descriptor === 'string') {
    return { requiredFeatures: [descriptor] };
  } else if (descriptor instanceof Array) {
    return {
      requiredFeatures: descriptor.filter((f) => f !== undefined)
    };
  } else {
    return descriptor ?? {};
  }
}







function mergeDeviceSelectionDescriptorIntoDeviceDescriptor(
src,
dst)
{
  const srcFixed = initUncanonicalizedDeviceDescriptor(src);
  if (srcFixed) {
    dst.requiredFeatures.push(...(srcFixed.requiredFeatures ?? []));
    Object.assign(dst.requiredLimits, srcFixed.requiredLimits ?? {});
  }
}

export class GPUTestSubcaseBatchState extends SubcaseBatchState {
  /** Provider for default device. */

  /** Provider for mismatched device. */

  /** The accumulated skip-if requirements for this subcase */
  skipIfRequirements = {
    requiredFeatures: [],
    requiredLimits: {},
    defaultQueue: {}
  };
  /** Whether or not to provide a mismatched device */
  useMismatchedDevice = false;

  async postInit() {
    // Skip all subcases if there's no device.
    await this.acquireProvider();
  }

  async finalize() {
    await super.finalize();

    // Ensure devicePool.release is called for both providers even if one rejects
    // and wait for both of them before proceeding.
    const results = await Promise.allSettled([
    this.provider?.then((x) => devicePool.release(x)),
    this.mismatchedProvider?.then((x) => mismatchedDevicePool.release(x))]
    );

    // If one of them rejected throw its reason. It should be an `Error`.
    for (const result of results) {
      if (result.status === 'rejected') throw result.reason;
    }
  }

  /** @internal MAINTENANCE_TODO: Make this not visible to test code? */
  acquireProvider() {
    if (this.provider === undefined) {
      this.requestDeviceWithRequiredParametersOrSkip(this.skipIfRequirements);
    }
    assert(this.provider !== undefined);
    assert(!this.useMismatchedDevice || this.mismatchedProvider !== undefined);
    return this.provider;
  }

  get isCompatibility() {
    return globalTestConfig.compatibility;
  }

  /**
   * Some tests or cases need particular feature flags or limits to be enabled.
   * Call this function with a descriptor or feature name (or `undefined`) to select a
   * GPUDevice with matching capabilities. If this isn't called, a default device is provided.
   *
   * If the request isn't supported, throws a SkipTestCase exception to skip the entire test case.
   */
  requestDeviceWithRequiredParametersOrSkip(
  descriptor,
  descriptorModifier)
  {
    assert(this.provider === undefined, "Can't selectDeviceOrSkipTestCase() multiple times");
    this.provider = devicePool.acquire(
      this.recorder,
      initUncanonicalizedDeviceDescriptor(descriptor),
      descriptorModifier
    );
    // Suppress uncaught promise rejection (we'll catch it later).
    this.provider.catch(() => {});

    if (this.useMismatchedDevice) {
      this.mismatchedProvider = mismatchedDevicePool.acquire(
        this.recorder,
        initUncanonicalizedDeviceDescriptor(descriptor),
        descriptorModifier
      );
      // Suppress uncaught promise rejection (we'll catch it later).
      this.mismatchedProvider.catch(() => {});
    }
  }

  /**
   * Some tests need a second device which is different from the first.
   * This requests a second device so it will be available during the test. If it is not called,
   * no second device will be available. The second device will be created with the
   * same features and limits as the first device.
   */
  usesMismatchedDevice() {
    assert(this.provider === undefined, 'Can not call usedMismatchedDevice after device creation');
    this.useMismatchedDevice = true;
  }

  /**
   * Some tests or cases need particular feature flags or limits to be enabled.
   * Call this function with a descriptor or feature name (or `undefined`) to add
   * features or limits required by the subcase. If the features or limits are not
   * available a SkipTestCase exception will be thrown to skip the entire test case.
   */
  selectDeviceOrSkipTestCase(descriptor) {
    mergeDeviceSelectionDescriptorIntoDeviceDescriptor(descriptor, this.skipIfRequirements);
  }

  /**
   * Convenience function for {@link selectDeviceOrSkipTestCase}.
   * Select a device with the features required by these texture format(s).
   * If the device creation fails, then skip the test case.
   */
  selectDeviceForTextureFormatOrSkipTestCase(
  formats)
  {
    if (!Array.isArray(formats)) {
      formats = [formats];
    }
    const features = new Set();
    for (const format of formats) {
      if (format !== undefined) {
        features.add(getRequiredFeatureForTextureFormat(format));
      }
    }

    this.selectDeviceOrSkipTestCase(Array.from(features));
  }

  /**
   * Convenience function for {@link selectDeviceOrSkipTestCase}.
   * Select a device with the features required by these query type(s).
   * If the device creation fails, then skip the test case.
   */
  selectDeviceForQueryTypeOrSkipTestCase(types) {
    if (!Array.isArray(types)) {
      types = [types];
    }
    const features = types.map((t) => kQueryTypeInfo[t].feature);
    this.selectDeviceOrSkipTestCase(features);
  }

  /** @internal MAINTENANCE_TODO: Make this not visible to test code? */
  acquireMismatchedProvider() {
    return this.mismatchedProvider;
  }

  skipIfCopyTextureToTextureNotSupportedForFormat(...formats) {
    if (this.isCompatibility) {
      for (const format of formats) {
        if (format && isCompressedTextureFormat(format)) {
          this.skip(`copyTextureToTexture with ${format} is not supported in compatibility mode`);
        }
      }
    }
  }

  /**
   * Skips test if the given interpolation type or sampling is not supported.
   */
  skipIfInterpolationTypeOrSamplingNotSupported({
    type,
    sampling



  }) {
    if (this.isCompatibility) {
      this.skipIf(
        type === 'linear',
        'interpolation type linear is not supported in compatibility mode'
      );
      this.skipIf(
        sampling === 'sample',
        'interpolation type linear is not supported in compatibility mode'
      );
      this.skipIf(
        type === 'flat' && (!sampling || sampling === 'first'),
        'interpolation type flat with sampling not set to either is not supported in compatibility mode'
      );
    }
  }

  /** Skips this test case if the `langFeature` is *not* supported. */
  skipIfLanguageFeatureNotSupported(langFeature) {
    if (!this.hasLanguageFeature(langFeature)) {
      this.skip(`WGSL language feature '${langFeature}' is not supported`);
    }
  }

  /** Skips this test case if the `langFeature` is supported. */
  skipIfLanguageFeatureSupported(langFeature) {
    if (this.hasLanguageFeature(langFeature)) {
      this.skip(`WGSL language feature '${langFeature}' is supported`);
    }
  }

  /** returns true iff the `langFeature` is supported  */
  hasLanguageFeature(langFeature) {
    const lf = getGPU(this.recorder).wgslLanguageFeatures;
    return lf !== undefined && lf.has(langFeature);
  }
}

/**
 * Base fixture for WebGPU tests.
 *
 * This class is a Fixture + a getter that returns a GPUDevice
 * as well as helpers that use that device.
 */
export class GPUTestBase extends Fixture {
  static MakeSharedState(
  recorder,
  params)
  {
    return new GPUTestSubcaseBatchState(recorder, params);
  }

  // This must be overridden in derived classes
  get device() {
    unreachable();
    return null;
  }

  /** GPUQueue for the test to use. (Same as `t.device.queue`.) */
  get queue() {
    return this.device.queue;
  }

  get isCompatibility() {
    return globalTestConfig.compatibility;
  }

  makeLimitVariant(limit, variant) {
    return makeValueTestVariant(this.device.limits[limit], variant);
  }

  canCallCopyTextureToBufferWithTextureFormat(format) {
    return !this.isCompatibility || !isCompressedTextureFormat(format);
  }

  /** Snapshot a GPUBuffer's contents, returning a new GPUBuffer with the `MAP_READ` usage. */
  createCopyForMapRead(src, srcOffset, size) {
    assert(srcOffset % 4 === 0);
    assert(size % 4 === 0);

    const dst = this.createBufferTracked({
      label: 'createCopyForMapRead',
      size,
      usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST
    });

    const c = this.device.createCommandEncoder({ label: 'createCopyForMapRead' });
    c.copyBufferToBuffer(src, srcOffset, dst, 0, size);
    this.queue.submit([c.finish()]);

    return dst;
  }

  /**
   * Offset and size passed to createCopyForMapRead must be divisible by 4. For that
   * we might need to copy more bytes from the buffer than we want to map.
   * begin and end values represent the part of the copied buffer that stores the contents
   * we initially wanted to map.
   * The copy will not cause an OOB error because the buffer size must be 4-aligned.
   */
  createAlignedCopyForMapRead(
  src,
  size,
  offset)
  {
    const alignedOffset = roundDown(offset, 4);
    const subarrayByteStart = offset - alignedOffset;
    const alignedSize = align(size + subarrayByteStart, 4);
    const mappable = this.createCopyForMapRead(src, alignedOffset, alignedSize);
    return { mappable, subarrayByteStart };
  }

  /**
   * Snapshot the current contents of a range of a GPUBuffer, and return them as a TypedArray.
   * Also provides a cleanup() function to unmap and destroy the staging buffer.
   */
  async readGPUBufferRangeTyped(
  src,
  {
    srcByteOffset = 0,
    method = 'copy',
    type,
    typedLength





  })
  {
    assert(
      srcByteOffset % type.BYTES_PER_ELEMENT === 0,
      'srcByteOffset must be a multiple of BYTES_PER_ELEMENT'
    );

    const byteLength = typedLength * type.BYTES_PER_ELEMENT;
    let mappable;
    let mapOffset, mapSize, subarrayByteStart;
    if (method === 'copy') {
      ({ mappable, subarrayByteStart } = this.createAlignedCopyForMapRead(
        src,
        byteLength,
        srcByteOffset
      ));
    } else if (method === 'map') {
      mappable = src;
      mapOffset = roundDown(srcByteOffset, 8);
      mapSize = align(byteLength, 4);
      subarrayByteStart = srcByteOffset - mapOffset;
    } else {
      unreachable();
    }

    assert(subarrayByteStart % type.BYTES_PER_ELEMENT === 0);
    const subarrayStart = subarrayByteStart / type.BYTES_PER_ELEMENT;

    // 2. Map the staging buffer, and create the TypedArray from it.
    await mappable.mapAsync(GPUMapMode.READ, mapOffset, mapSize);
    const mapped = new type(mappable.getMappedRange(mapOffset, mapSize));
    const data = mapped.subarray(subarrayStart, typedLength);

    return {
      data,
      cleanup() {
        mappable.unmap();
        mappable.destroy();
      }
    };
  }

  /**
   * Skips test if device does not have feature.
   * Note: Try to use one of the more specific skipIf tests if possible.
   */
  skipIfDeviceDoesNotHaveFeature(feature) {
    this.skipIf(!this.device.features.has(feature), `device does not have feature: '${feature}'`);
  }

  /**
   * Skips test if device des not support query type.
   */
  skipIfDeviceDoesNotSupportQueryType(...types) {
    for (const type of types) {
      const feature = kQueryTypeInfo[type].feature;
      if (feature) {
        this.skipIfDeviceDoesNotHaveFeature(feature);
      }
    }
  }

  skipIfDepthTextureCanNotBeUsedWithNonComparisonSampler() {
    this.skipIf(
      this.isCompatibility,
      'depth textures are not usable with non-comparison samplers in compatibility mode'
    );
  }

  /**
   * Skips test if any format is not supported.
   */
  skipIfTextureFormatNotSupported(...formats) {
    for (const format of formats) {
      if (!format) {
        continue;
      }
      if (format === 'bgra8unorm-srgb') {
        if (isCompatibilityDevice(this.device)) {
          this.skip(`texture format '${format}' is not supported`);
        }
      }
      const feature = getRequiredFeatureForTextureFormat(format);
      this.skipIf(
        !!feature && !this.device.features.has(feature),
        `texture format '${format}' requires feature: '${feature}'`
      );
    }
  }

  skipIfTextureFormatAndViewDimensionNotCompatible(
  format,
  viewDimension)
  {
    this.skipIf(
      !textureViewDimensionAndFormatCompatibleForDevice(this.device, viewDimension, format),
      `format: ${format} does not support viewDimension: ${viewDimension}`
    );
  }

  skipIfTextureFormatAndDimensionNotCompatible(
  format,
  dimension)
  {
    this.skipIf(
      !textureDimensionAndFormatCompatibleForDevice(this.device, dimension, format),
      `format: ${format} does not support dimension: ${dimension}`
    );
  }

  skipIfTextureFormatNotResolvable(...formats) {
    for (const format of formats) {
      if (format === undefined) continue;
      if (!isTextureFormatResolvable(this.device, format)) {
        this.skip(`texture format '${format}' is not resolvable`);
      }
    }
  }

  skipIfTextureViewDimensionNotSupported(...dimensions) {
    if (isCompatibilityDevice(this.device)) {
      for (const dimension of dimensions) {
        if (dimension === 'cube-array') {
          this.skip(`texture view dimension '${dimension}' is not supported`);
        }
      }
    }
  }

  skipIfCopyTextureToTextureNotSupportedForFormat(...formats) {
    if (isCompatibilityDevice(this.device)) {
      for (const format of formats) {
        if (format && isCompressedTextureFormat(format)) {
          this.skip(`copyTextureToTexture with ${format} is not supported`);
        }
      }
    }
  }

  skipIfTextureLoadNotSupportedForTextureType(...types) {
    if (this.isCompatibility) {
      for (const type of types) {
        switch (type) {
          case 'texture_depth_2d':
          case 'texture_depth_2d_array':
          case 'texture_depth_multisampled_2d':
            this.skip(`${type} is not supported by textureLoad in compatibility mode`);
        }
      }
    }
  }

  skipIfTextureFormatNotUsableWithStorageAccessMode(
  access,
  ...formats)
  {
    for (const format of formats) {
      if (!format) continue;

      if (!isTextureFormatUsableWithStorageAccessMode(this.device, format, access)) {
        this.skip(
          `Texture with ${format} is not usable as a storage texture with access ${access}`
        );
      }
    }
  }

  skipIfTextureFormatNotUsableAsRenderAttachment(...formats) {
    for (const format of formats) {
      if (format && !isTextureFormatUsableAsRenderAttachment(this.device, format)) {
        this.skip(`Texture with ${format} is not usable as a render attachment`);
      }
    }
  }

  skipIfTextureFormatNotMultisampled(...formats) {
    for (const format of formats) {
      if (format === undefined) continue;
      if (!isTextureFormatMultisampled(this.device, format)) {
        this.skip(`texture format '${format}' does not support multisampling`);
      }
    }
  }

  skipIfTextureFormatNotBlendable(...formats) {
    for (const format of formats) {
      if (format === undefined) continue;
      this.skipIf(isSintOrUintFormat(format), 'sint/uint formats are not blendable');
      if (is32Float(format)) {
        this.skipIf(
          !this.device.features.has('float32-blendable'),
          `texture format '${format}' is not blendable`
        );
      }
    }
  }

  skipIfTextureFormatNotFilterable(...formats) {
    for (const format of formats) {
      if (format === undefined) continue;
      this.skipIf(isSintOrUintFormat(format), 'sint/uint formats are not filterable');
      if (is32Float(format)) {
        this.skipIf(
          !this.device.features.has('float32-filterable'),
          `texture format '${format}' is not filterable`
        );
      }
    }
  }

  skipIfTextureFormatDoesNotSupportUsage(
  usage,
  ...formats)
  {
    for (const format of formats) {
      if (!format) continue;
      if (usage & GPUTextureUsage.RENDER_ATTACHMENT) {
        this.skipIfTextureFormatNotUsableAsRenderAttachment(format);
      }
      if (usage & GPUTextureUsage.STORAGE_BINDING) {
        this.skipIfTextureFormatNotUsableWithStorageAccessMode('write-only', format);
      }
    }
  }

  skipIfTextureFormatDoesNotSupportCopyTextureToBuffer(format) {
    this.skipIf(
      !this.canCallCopyTextureToBufferWithTextureFormat(format),
      `can not use copyTextureToBuffer with ${format}`
    );
  }

  skipIfTextureFormatPossiblyNotUsableWithCopyExternalImageToTexture(format) {
    this.skipIf(
      !isTextureFormatUsableWithCopyExternalImageToTexture(this.device, format),
      `can not use copyExternalImageToTexture with ${format}`
    );
  }

  /** Skips this test case if the `langFeature` is *not* supported. */
  skipIfLanguageFeatureNotSupported(langFeature) {
    if (!this.hasLanguageFeature(langFeature)) {
      this.skip(`WGSL language feature '${langFeature}' is not supported`);
    }
  }

  /** Skips this test case if the `langFeature` is supported. */
  skipIfLanguageFeatureSupported(langFeature) {
    if (this.hasLanguageFeature(langFeature)) {
      this.skip(`WGSL language feature '${langFeature}' is supported`);
    }
  }

  /** returns true if the `langFeature` is supported  */
  hasLanguageFeature(langFeature) {
    const lf = getGPU(this.rec).wgslLanguageFeatures;
    return lf !== undefined && lf.has(langFeature);
  }

  /**
   * Expect a GPUBuffer's contents to pass the provided check.
   *
   * A library of checks can be found in {@link webgpu/util/check_contents}.
   */
  expectGPUBufferValuesPassCheck(
  src,
  check,
  {
    srcByteOffset = 0,
    type,
    typedLength,
    method = 'copy',
    mode = 'fail'






  })
  {
    const readbackPromise = this.readGPUBufferRangeTyped(src, {
      srcByteOffset,
      type,
      typedLength,
      method
    });
    this.eventualAsyncExpectation(async (niceStack) => {
      const readback = await readbackPromise;
      this.expectOK(check(readback.data), { mode, niceStack });
      readback.cleanup();
    });
  }

  /**
   * Expect a GPUBuffer's contents to equal the values in the provided TypedArray.
   */
  expectGPUBufferValuesEqual(
  src,
  expected,
  srcByteOffset = 0,
  { method = 'copy', mode = 'fail' } = {})
  {
    this.expectGPUBufferValuesPassCheck(src, (a) => checkElementsEqual(a, expected), {
      srcByteOffset,
      type: expected.constructor,
      typedLength: expected.length,
      method,
      mode
    });
  }

  /**
   * Expect a buffer to consist exclusively of rows of some repeated expected value. The size of
   * `expectedValue` must be 1, 2, or any multiple of 4 bytes. Rows in the buffer are expected to be
   * zero-padded out to `bytesPerRow`. `minBytesPerRow` is the number of bytes per row that contain
   * actual (non-padding) data and must be an exact multiple of the byte-length of `expectedValue`.
   */
  expectGPUBufferRepeatsSingleValue(
  buffer,
  {
    expectedValue,
    numRows,
    minBytesPerRow,
    bytesPerRow





  })
  {
    const valueSize = expectedValue.byteLength;
    assert(valueSize === 1 || valueSize === 2 || valueSize % 4 === 0);
    assert(minBytesPerRow % valueSize === 0);
    assert(bytesPerRow % 4 === 0);

    // If the buffer is small enough, just generate the full expected buffer contents and check
    // against them on the CPU.
    const kMaxBufferSizeToCheckOnCpu = 256 * 1024;
    const bufferSize = bytesPerRow * (numRows - 1) + minBytesPerRow;
    if (bufferSize <= kMaxBufferSizeToCheckOnCpu) {
      const valueBytes = Array.from(new Uint8Array(expectedValue));
      const rowValues = new Array(minBytesPerRow / valueSize).fill(valueBytes);
      const rowBytes = new Uint8Array([].concat(...rowValues));
      const expectedContents = new Uint8Array(bufferSize);
      range(numRows, (row) => expectedContents.set(rowBytes, row * bytesPerRow));
      this.expectGPUBufferValuesEqual(buffer, expectedContents);
      return;
    }

    // Copy into a buffer suitable for STORAGE usage.
    const storageBuffer = this.createBufferTracked({
      label: 'expectGPUBufferRepeatsSingleValue:storageBuffer',
      size: bufferSize,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST
    });

    // This buffer conveys the data we expect to see for a single value read. Since we read 32 bits at
    // a time, for values smaller than 32 bits we pad this expectation with repeated value data, or
    // with zeroes if the width of a row in the buffer is less than 4 bytes. For value sizes larger
    // than 32 bits, we assume they're a multiple of 32 bits and expect to read exact matches of
    // `expectedValue` as-is.
    const expectedDataSize = Math.max(4, valueSize);
    const expectedDataBuffer = this.createBufferTracked({
      label: 'expectGPUBufferRepeatsSingleValue:expectedDataBuffer',
      size: expectedDataSize,
      usage: GPUBufferUsage.STORAGE,
      mappedAtCreation: true
    });
    const expectedData = new Uint32Array(expectedDataBuffer.getMappedRange());
    if (valueSize === 1) {
      const value = new Uint8Array(expectedValue)[0];
      const values = new Array(Math.min(4, minBytesPerRow)).fill(value);
      const padding = new Array(Math.max(0, 4 - values.length)).fill(0);
      const expectedBytes = new Uint8Array(expectedData.buffer);
      expectedBytes.set([...values, ...padding]);
    } else if (valueSize === 2) {
      const value = new Uint16Array(expectedValue)[0];
      const expectedWords = new Uint16Array(expectedData.buffer);
      expectedWords.set([value, minBytesPerRow > 2 ? value : 0]);
    } else {
      expectedData.set(new Uint32Array(expectedValue));
    }
    expectedDataBuffer.unmap();

    // The output buffer has one 32-bit entry per buffer row. An entry's value will be 1 if every
    // read from the corresponding row matches the expected data derived above, or 0 otherwise.
    const resultBuffer = this.createBufferTracked({
      label: 'expectGPUBufferRepeatsSingleValue:resultBuffer',
      size: numRows * 4,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
    });

    const readsPerRow = Math.ceil(minBytesPerRow / expectedDataSize);
    const reducer = `
    struct Buffer { data: array<u32>, };
    @group(0) @binding(0) var<storage, read> expected: Buffer;
    @group(0) @binding(1) var<storage, read> in: Buffer;
    @group(0) @binding(2) var<storage, read_write> out: Buffer;
    @compute @workgroup_size(1) fn reduce(
        @builtin(global_invocation_id) id: vec3<u32>) {
      let rowBaseIndex = id.x * ${bytesPerRow / 4}u;
      let readSize = ${expectedDataSize / 4}u;
      out.data[id.x] = 1u;
      for (var i: u32 = 0u; i < ${readsPerRow}u; i = i + 1u) {
        let elementBaseIndex = rowBaseIndex + i * readSize;
        for (var j: u32 = 0u; j < readSize; j = j + 1u) {
          if (in.data[elementBaseIndex + j] != expected.data[j]) {
            out.data[id.x] = 0u;
            return;
          }
        }
      }
    }
    `;

    const pipeline = this.device.createComputePipeline({
      layout: 'auto',
      compute: {
        module: this.device.createShaderModule({ code: reducer }),
        entryPoint: 'reduce'
      }
    });

    const bindGroup = this.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
      { binding: 0, resource: { buffer: expectedDataBuffer } },
      { binding: 1, resource: { buffer: storageBuffer } },
      { binding: 2, resource: { buffer: resultBuffer } }]

    });

    const commandEncoder = this.device.createCommandEncoder({
      label: 'expectGPUBufferRepeatsSingleValue'
    });
    commandEncoder.copyBufferToBuffer(buffer, 0, storageBuffer, 0, bufferSize);
    const pass = commandEncoder.beginComputePass();
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.dispatchWorkgroups(numRows);
    pass.end();
    this.device.queue.submit([commandEncoder.finish()]);

    const expectedResults = new Array(numRows).fill(1);
    this.expectGPUBufferValuesEqual(resultBuffer, new Uint32Array(expectedResults));
  }

  // MAINTENANCE_TODO: add an expectContents for textures, which logs data: uris on failure

  /**
   * Expect an entire GPUTexture to have a single color at the given mip level (defaults to 0).
   * MAINTENANCE_TODO: Remove this and/or replace it with a helper in TextureTestMixin.
   */
  expectSingleColor(
  src,
  format,
  {
    size,
    exp,
    dimension = '2d',
    slice = 0,
    layout






  })
  {
    assert(
      slice === 0 || dimension === '2d',
      'texture slices are only implemented for 2d textures'
    );

    format = resolvePerAspectFormat(format, layout?.aspect);
    const { byteLength, minBytesPerRow, bytesPerRow, rowsPerImage, mipSize } = getTextureCopyLayout(
      format,
      dimension,
      size,
      layout
    );
    // MAINTENANCE_TODO: getTextureCopyLayout does not return the proper size for array textures,
    // i.e. it will leave the z/depth value as is instead of making it 1 when dealing with 2d
    // texture arrays. Since we are passing in the dimension, we should update it to return the
    // corrected size.
    const copySize = [
    mipSize[0],
    dimension !== '1d' ? mipSize[1] : 1,
    dimension === '3d' ? mipSize[2] : 1];


    const rep = kTexelRepresentationInfo[format];
    const expectedTexelData = rep.pack(rep.encode(exp));

    const buffer = this.createBufferTracked({
      label: 'expectSingleColor',
      size: byteLength,
      usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
    });

    const commandEncoder = this.device.createCommandEncoder({ label: 'expectSingleColor' });
    commandEncoder.copyTextureToBuffer(
      {
        texture: src,
        mipLevel: layout?.mipLevel,
        origin: { x: 0, y: 0, z: slice },
        aspect: layout?.aspect
      },
      { buffer, bytesPerRow, rowsPerImage },
      copySize
    );
    this.queue.submit([commandEncoder.finish()]);

    this.expectGPUBufferRepeatsSingleValue(buffer, {
      expectedValue: expectedTexelData,
      numRows: rowsPerImage * copySize[2],
      minBytesPerRow,
      bytesPerRow
    });
  }

  /**
   * Return a GPUBuffer that data are going to be written into.
   * MAINTENANCE_TODO: Remove this once expectSinglePixelBetweenTwoValuesIn2DTexture is removed.
   */
  readSinglePixelFrom2DTexture(
  src,
  format,
  { x, y },
  { slice = 0, layout })
  {
    const { byteLength, bytesPerRow, rowsPerImage } = getTextureSubCopyLayout(
      format,
      [1, 1],
      layout
    );
    const buffer = this.createBufferTracked({
      label: 'readSinglePixelFrom2DTexture',
      size: byteLength,
      usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
    });

    const commandEncoder = this.device.createCommandEncoder({
      label: 'readSinglePixelFrom2DTexture'
    });
    commandEncoder.copyTextureToBuffer(
      { texture: src, mipLevel: layout?.mipLevel, origin: { x, y, z: slice } },
      { buffer, bytesPerRow, rowsPerImage },
      [1, 1]
    );
    this.queue.submit([commandEncoder.finish()]);

    return buffer;
  }

  /**
   * Take a single pixel of a 2D texture, interpret it using a TypedArray of the `expected` type,
   * and expect each value in that array to be between the corresponding "expected" values
   * (either `a[i] <= actual[i] <= b[i]` or `a[i] >= actual[i] => b[i]`).
   * MAINTENANCE_TODO: Remove this once there is a way to deal with undefined lerp-ed values.
   */
  expectSinglePixelBetweenTwoValuesIn2DTexture(
  src,
  format,
  { x, y },
  {
    exp,
    slice = 0,
    layout,
    generateWarningOnly = false,
    checkElementsBetweenFn = (act, [a, b]) =>
    checkElementsBetween(act, [(i) => a[i], (i) => b[i]])









  })
  {
    assert(exp[0].constructor === exp[1].constructor);
    const constructor = exp[0].constructor;
    assert(exp[0].length === exp[1].length);
    const typedLength = exp[0].length;

    const buffer = this.readSinglePixelFrom2DTexture(src, format, { x, y }, { slice, layout });
    this.expectGPUBufferValuesPassCheck(buffer, (a) => checkElementsBetweenFn(a, exp), {
      type: constructor,
      typedLength,
      mode: generateWarningOnly ? 'warn' : 'fail'
    });
  }

  /**
   * Emulate a texture to buffer copy by using a compute shader
   * to load texture values of a subregion of a 2d texture and write to a storage buffer.
   * For sample count == 1, the buffer contains extent[0] * extent[1] of the sample.
   * For sample count > 1, the buffer contains extent[0] * extent[1] * (N = sampleCount) values sorted
   * in the order of their sample index [0, sampleCount - 1]
   *
   * This can be useful when the texture to buffer copy is not available to the texture format
   * e.g. (depth24plus), or when the texture is multisampled.
   *
   * MAINTENANCE_TODO: extend texture dimension to 1d and 3d.
   *
   * @returns storage buffer containing the copied value from the texture.
   */
  copy2DTextureToBufferUsingComputePass(
  type,
  componentCount,
  textureView,
  sampleCount = 1,
  extent_ = [1, 1, 1],
  origin_ = [0, 0, 0])
  {
    const origin = reifyOrigin3D(origin_);
    const extent = reifyExtent3D(extent_);
    const width = extent.width;
    const height = extent.height;
    const kWorkgroupSizeX = 8;
    const kWorkgroupSizeY = 8;
    const textureSrcCode =
    sampleCount === 1 ?
    `@group(0) @binding(0) var src: texture_2d<${type}>;` :
    `@group(0) @binding(0) var src: texture_multisampled_2d<${type}>;`;
    const code = `
      struct Buffer {
        data: array<${type}>,
      };

      ${textureSrcCode}
      @group(0) @binding(1) var<storage, read_write> dst : Buffer;

      struct Params {
        origin: vec2u,
        extent: vec2u,
      };
      @group(0) @binding(2) var<uniform> params : Params;

      @compute @workgroup_size(${kWorkgroupSizeX}, ${kWorkgroupSizeY}, 1) fn main(@builtin(global_invocation_id) id : vec3u) {
        let boundary = params.origin + params.extent;
        let coord = params.origin + id.xy;
        if (any(coord >= boundary)) {
          return;
        }
        let offset = (id.x + id.y * params.extent.x) * ${componentCount} * ${sampleCount};
        for (var sampleIndex = 0u; sampleIndex < ${sampleCount};
          sampleIndex = sampleIndex + 1) {
          let o = offset + sampleIndex * ${componentCount};
          let v = textureLoad(src, coord.xy, sampleIndex);
          for (var component = 0u; component < ${componentCount}; component = component + 1) {
            dst.data[o + component] = v[component];
          }
        }
      }
    `;
    const computePipeline = this.device.createComputePipeline({
      layout: 'auto',
      compute: {
        module: this.device.createShaderModule({
          code
        }),
        entryPoint: 'main'
      }
    });

    const storageBuffer = this.createBufferTracked({
      label: 'copy2DTextureToBufferUsingComputePass:storageBuffer',
      size: sampleCount * type.size * componentCount * width * height,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST | GPUBufferUsage.COPY_SRC
    });

    const uniformBuffer = this.makeBufferWithContents(
      new Uint32Array([origin.x, origin.y, width, height]),
      GPUBufferUsage.UNIFORM
    );

    const uniformBindGroup = this.device.createBindGroup({
      layout: computePipeline.getBindGroupLayout(0),
      entries: [
      {
        binding: 0,
        resource: textureView
      },
      {
        binding: 1,
        resource: {
          buffer: storageBuffer
        }
      },
      {
        binding: 2,
        resource: {
          buffer: uniformBuffer
        }
      }]

    });

    const encoder = this.device.createCommandEncoder({
      label: 'copy2DTextureToBufferUsingComputePass'
    });
    const pass = encoder.beginComputePass();
    pass.setPipeline(computePipeline);
    pass.setBindGroup(0, uniformBindGroup);
    pass.dispatchWorkgroups(
      Math.floor((width + kWorkgroupSizeX - 1) / kWorkgroupSizeX),
      Math.floor((height + kWorkgroupSizeY - 1) / kWorkgroupSizeY),
      1
    );
    pass.end();
    this.device.queue.submit([encoder.finish()]);

    return storageBuffer;
  }

  /**
   * Expect the specified WebGPU error to be generated when running the provided function.
   */
  expectGPUError(filter, fn, shouldError = true) {
    // If no error is expected, we let the scope surrounding the test catch it.
    if (!shouldError) {
      return fn();
    }

    this.device.pushErrorScope(filter);
    const returnValue = fn();
    const promise = this.device.popErrorScope();

    this.eventualAsyncExpectation(async (niceStack) => {
      const error = await promise;

      let failed = false;
      switch (filter) {
        case 'out-of-memory':
          failed = !(error instanceof GPUOutOfMemoryError);
          break;
        case 'validation':
          failed = !(error instanceof GPUValidationError);
          break;
      }

      if (failed) {
        niceStack.message = `Expected ${filter} error`;
        this.rec.expectationFailed(niceStack);
      } else {
        niceStack.message = `Captured ${filter} error`;
        if (error instanceof GPUValidationError) {
          niceStack.message += ` - ${error.message}`;
        }
        this.rec.debug(niceStack);
      }
    });

    return returnValue;
  }

  /**
   * Expect a validation error inside the callback.
   *
   * Tests should always do just one WebGPU call in the callback, to make sure that's what's tested.
   */
  expectValidationError(fn, shouldError = true) {
    // If no error is expected, we let the scope surrounding the test catch it.
    if (shouldError) {
      this.device.pushErrorScope('validation');
    }

    // Note: A return value is not allowed for the callback function. This is to avoid confusion
    // about what the actual behavior would be; either of the following could be reasonable:
    //   - Make expectValidationError async, and have it await on fn(). This causes an async split
    //     between pushErrorScope and popErrorScope, so if the caller doesn't `await` on
    //     expectValidationError (either accidentally or because it doesn't care to do so), then
    //     other test code will be (nondeterministically) caught by the error scope.
    //   - Make expectValidationError NOT await fn(), but just execute its first block (until the
    //     first await) and return the return value (a Promise). This would be confusing because it
    //     would look like the error scope includes the whole async function, but doesn't.
    // If we do decide we need to return a value, we should use the latter semantic.
    const returnValue = fn();
    assert(
      returnValue === undefined,
      'expectValidationError callback should not return a value (or be async)'
    );

    if (shouldError) {
      const promise = this.device.popErrorScope();

      this.eventualAsyncExpectation(async (niceStack) => {
        const gpuValidationError = await promise;
        if (!gpuValidationError) {
          niceStack.message = 'Validation succeeded unexpectedly.';
          this.rec.validationFailed(niceStack);
        } else if (gpuValidationError instanceof GPUValidationError) {
          niceStack.message = `Validation failed, as expected - ${gpuValidationError.message}`;
          this.rec.debug(niceStack);
        }
      });
    }
  }

  /**
   * Expect a validation error or exception inside the callback.
   *
   * Tests should always do just one WebGPU call in the callback, to make sure that's what's tested.
   */
  expectValidationErrorOrException(
  fn,
  shouldError = true,
  shouldThrow = true)
  {
    if (shouldThrow) {
      this.shouldThrow(shouldError, fn);
    } else {
      this.expectValidationError(fn, shouldError);
    }
  }

  /** Create a GPUBuffer and track it for cleanup at the end of the test. */
  createBufferTracked(descriptor) {
    return this.trackForCleanup(this.device.createBuffer(descriptor));
  }

  /** Create a GPUTexture and track it for cleanup at the end of the test. */
  createTextureTracked(descriptor) {
    return this.trackForCleanup(this.device.createTexture(descriptor));
  }

  /** Create a GPUQuerySet and track it for cleanup at the end of the test. */
  createQuerySetTracked(descriptor) {
    return this.trackForCleanup(this.device.createQuerySet(descriptor));
  }

  /**
   * Creates a buffer with the contents of some TypedArray.
   * The buffer size will always be aligned to 4 as we set mappedAtCreation === true when creating the
   * buffer.
   *
   * MAINTENANCE_TODO: Several call sites would be simplified if this took ArrayBuffer as well.
   */
  makeBufferWithContents(dataArray, usage) {
    const buffer = this.createBufferTracked({
      mappedAtCreation: true,
      size: align(dataArray.byteLength, 4),
      usage
    });
    memcpy({ src: dataArray }, { dst: buffer.getMappedRange() });
    buffer.unmap();
    return buffer;
  }

  /**
   * Returns a GPUCommandEncoder, GPUComputePassEncoder, GPURenderPassEncoder, or
   * GPURenderBundleEncoder, and a `finish` method returning a GPUCommandBuffer.
   * Allows testing methods which have the same signature across multiple encoder interfaces.
   *
   * @example
   * ```
   * g.test('popDebugGroup')
   *   .params(u => u.combine('encoderType', kEncoderTypes))
   *   .fn(t => {
   *     const { encoder, finish } = t.createEncoder(t.params.encoderType);
   *     encoder.popDebugGroup();
   *   });
   *
   * g.test('writeTimestamp')
   *   .params(u => u.combine('encoderType', ['non-pass', 'compute pass', 'render pass'] as const)
   *   .fn(t => {
   *     const { encoder, finish } = t.createEncoder(t.params.encoderType);
   *     // Encoder type is inferred, so `writeTimestamp` can be used even though it doesn't exist
   *     // on GPURenderBundleEncoder.
   *     encoder.writeTimestamp(args);
   *   });
   * ```
   */
  createEncoder(
  encoderType,
  {
    attachmentInfo,
    occlusionQuerySet,
    targets




  } = {})
  {
    const fullAttachmentInfo = {
      // Defaults if not overridden:
      colorFormats: ['rgba8unorm'],
      sampleCount: 1,
      // Passed values take precedent.
      ...attachmentInfo
    };

    switch (encoderType) {
      case 'non-pass':{
          const encoder = this.device.createCommandEncoder();

          return new CommandBufferMaker(this, encoder, () => {
            return encoder.finish();
          });
        }
      case 'render bundle':{
          const device = this.device;
          const rbEncoder = device.createRenderBundleEncoder(fullAttachmentInfo);
          const pass = this.createEncoder('render pass', { attachmentInfo, targets });

          return new CommandBufferMaker(this, rbEncoder, () => {
            pass.encoder.executeBundles([rbEncoder.finish()]);
            return pass.finish();
          });
        }
      case 'compute pass':{
          const commandEncoder = this.device.createCommandEncoder();
          const encoder = commandEncoder.beginComputePass();

          return new CommandBufferMaker(this, encoder, () => {
            encoder.end();
            return commandEncoder.finish();
          });
        }
      case 'render pass':{
          const makeAttachmentView = (format) =>
          this.createTextureTracked({
            size: [16, 16, 1],
            format,
            usage: GPUTextureUsage.RENDER_ATTACHMENT,
            sampleCount: fullAttachmentInfo.sampleCount
          }).createView();

          let depthStencilAttachment = undefined;
          if (fullAttachmentInfo.depthStencilFormat !== undefined) {
            depthStencilAttachment = {
              view: makeAttachmentView(fullAttachmentInfo.depthStencilFormat),
              depthReadOnly: fullAttachmentInfo.depthReadOnly,
              stencilReadOnly: fullAttachmentInfo.stencilReadOnly
            };
            if (
            isDepthTextureFormat(fullAttachmentInfo.depthStencilFormat) &&
            !fullAttachmentInfo.depthReadOnly)
            {
              depthStencilAttachment.depthClearValue = 0;
              depthStencilAttachment.depthLoadOp = 'clear';
              depthStencilAttachment.depthStoreOp = 'discard';
            }
            if (
            isStencilTextureFormat(fullAttachmentInfo.depthStencilFormat) &&
            !fullAttachmentInfo.stencilReadOnly)
            {
              depthStencilAttachment.stencilClearValue = 1;
              depthStencilAttachment.stencilLoadOp = 'clear';
              depthStencilAttachment.stencilStoreOp = 'discard';
            }
          }
          const passDesc = {
            colorAttachments: Array.from(fullAttachmentInfo.colorFormats, (format, i) =>
            format ?
            {
              view: targets ? targets[i] : makeAttachmentView(format),
              clearValue: [0, 0, 0, 0],
              loadOp: 'clear',
              storeOp: 'store'
            } :
            null
            ),
            depthStencilAttachment,
            occlusionQuerySet
          };

          const commandEncoder = this.device.createCommandEncoder();
          const encoder = commandEncoder.beginRenderPass(passDesc);
          return new CommandBufferMaker(this, encoder, () => {
            encoder.end();
            return commandEncoder.finish();
          });
        }
    }
    unreachable();
  }
}

/**
 * Fixture for WebGPU tests that uses a DeviceProvider
 */
export class GPUTest extends GPUTestBase {
  // Should never be undefined in a test. If it is, init() must not have run/finished.



  async init() {
    await super.init();

    this.provider = await this.sharedState.acquireProvider();
    this.mismatchedProvider = await this.sharedState.acquireMismatchedProvider();
  }

  /** GPUAdapter that the device was created from. */
  get adapter() {
    assert(this.provider !== undefined, 'internal error: DeviceProvider missing');
    return this.provider.adapter;
  }

  /**
   * GPUDevice for the test to use.
   */
  get device() {
    assert(this.provider !== undefined, 'internal error: DeviceProvider missing');
    return this.provider.device;
  }

  /**
   * GPUDevice for tests requiring a second device different from the default one,
   * e.g. for creating objects for by device_mismatch validation tests.
   */
  get mismatchedDevice() {
    assert(
      this.mismatchedProvider !== undefined,
      'usesMismatchedDevice or selectMismatchedDeviceOrSkipTestCase was not called in beforeAllSubcases'
    );
    return this.mismatchedProvider.device;
  }

  /**
   * Expects that the device should be lost for a particular reason at the teardown of the test.
   */
  expectDeviceLost(reason) {
    assert(this.provider !== undefined, 'internal error: GPUDevice missing?');
    this.provider.expectDeviceLost(reason);
  }
}

/**
 * Gets the adapter limits as a standard JavaScript object.
 */
function getAdapterLimitsAsDeviceRequiredLimits(adapter) {
  const requiredLimits = {};
  const adapterLimits = adapter.limits;
  for (const key in adapter.limits) {
    // MAINTENANCE_TODO: Remove this once minSubgroupSize is removed from
    // chromium.
    if (key === 'maxSubgroupSize' || key === 'minSubgroupSize') {
      continue;
    }
    requiredLimits[key] = adapterLimits[key];
  }
  return requiredLimits;
}

/**
 * Removes limits that don't exist on the adapter.
 * A test might request a new limit that not all implementations support. The test itself
 * should check the requested limit using code that expects undefined.
 *
 * ```ts
 *    t.skipIf(limit < 2);     // BAD! Doesn't skip if unsupported because undefined is never less than 2.
 *    t.skipIf(!(limit >= 2)); // Good. Skips if limits is not >= 2. undefined is not >= 2.
 * ```
 */
function removeNonExistentLimits(adapter, limits) {
  const filteredLimits = {};
  const adapterLimits = adapter.limits;
  for (const [limit, value] of Object.entries(limits)) {
    if (adapterLimits[limit] !== undefined) {
      filteredLimits[limit] = value;
    }
  }
  return filteredLimits;
}

function applyLimitsToDescriptor(
adapter,
desc,
getRequiredLimits)
{
  const descWithMaxLimits = {
    requiredFeatures: [],
    defaultQueue: {},
    ...desc,
    requiredLimits: removeNonExistentLimits(adapter, getRequiredLimits(adapter))
  };
  return descWithMaxLimits;
}

function getAdapterFeaturesAsDeviceRequiredFeatures(adapter) {
  return [...adapter.features].filter(
    (f) => f !== 'core-features-and-limits'
  );
}

function applyFeaturesToDescriptor(
adapter,
desc,
getRequiredFeatures)
{
  const existingRequiredFeatures = (desc && desc?.requiredFeatures) ?? [];
  const descWithRequiredFeatures = {
    requiredLimits: {},
    defaultQueue: {},
    ...desc,
    requiredFeatures: [...existingRequiredFeatures, ...getRequiredFeatures(adapter)]
  };
  return descWithRequiredFeatures;
}

/**
 * Used by RequiredLimitsTestMixin to allow you to request specific limits
 *
 * Supply a `getRequiredLimits` function that given a GPUAdapter, turns the limits
 * you want.
 *
 * Also supply a key function that returns a device key. You should generally return
 * the name of each limit you request and any math you did on the limit. For example
 *
 * ```js
 * {
 *   getRequiredLimits(adapter) {
 *     return {
 *       maxBindGroups: adapter.limits.maxBindGroups / 2,
 *       maxTextureDimensions2D: Math.max(adapter.limits.maxTextureDimensions2D, 8192),
 *     },
 *   },
 *   key() {
 *     return `
 *       maxBindGroups / 2,
 *       max(maxTextureDimension2D, 8192),
 *     `;
 *   },
 * }
 * ```
 *
 * Its important to note, the key is used BEFORE knowing the adapter limits to get a device
 * that was already created with the same key.
 */





/**
 * Used by RequiredLimitsTest to request a device with all requested limits of the adapter.
 */
export class RequiredLimitsGPUTestSubcaseBatchState extends GPUTestSubcaseBatchState {

  constructor(
  recorder,
  params,
  requiredLimitsHelper)
  {
    super(recorder, params);this.recorder = recorder;this.params = params;
    this.requiredLimitsHelper = requiredLimitsHelper;
  }
  requestDeviceWithRequiredParametersOrSkip(
  descriptor,
  descriptorModifier)
  {
    const requiredLimitsHelper = this.requiredLimitsHelper;
    const mod = {
      descriptorModifier(adapter, desc) {
        desc = descriptorModifier?.descriptorModifier ?
        descriptorModifier.descriptorModifier(adapter, desc) :
        desc;
        return applyLimitsToDescriptor(adapter, desc, requiredLimitsHelper.getRequiredLimits);
      },
      keyModifier(baseKey) {
        return `${baseKey}:${requiredLimitsHelper.key()}`;
      }
    };
    super.requestDeviceWithRequiredParametersOrSkip(
      initUncanonicalizedDeviceDescriptor(descriptor),
      mod
    );
  }
}





/**
 * A text mixin to make it relatively easy to request specific limits.
 */
export function RequiredLimitsTestMixin(
Base,
requiredLimitsHelper)
{
  class RequiredLimitsImpl extends
  Base

  {
    //
    static MakeSharedState(
    recorder,
    params)
    {
      return new RequiredLimitsGPUTestSubcaseBatchState(recorder, params, requiredLimitsHelper);
    }
  }

  return RequiredLimitsImpl;
}

/**
 * Used by AllFeaturesMaxLimitsGPUTest to request a device with all limits and features of the adapter.
 */
export class AllFeaturesMaxLimitsGPUTestSubcaseBatchState extends GPUTestSubcaseBatchState {
  constructor(
  recorder,
  params)
  {
    super(recorder, params);this.recorder = recorder;this.params = params;
  }
  requestDeviceWithRequiredParametersOrSkip(
  descriptor,
  descriptorModifier)
  {
    const mod = {
      descriptorModifier(adapter, desc) {
        desc = descriptorModifier?.descriptorModifier ?
        descriptorModifier.descriptorModifier(adapter, desc) :
        desc;
        desc = applyLimitsToDescriptor(adapter, desc, getAdapterLimitsAsDeviceRequiredLimits);
        desc = applyFeaturesToDescriptor(adapter, desc, getAdapterFeaturesAsDeviceRequiredFeatures);
        return desc;
      },
      keyModifier(baseKey) {
        return `${baseKey}:AllFeaturesMaxLimits`;
      }
    };
    super.requestDeviceWithRequiredParametersOrSkip(
      initUncanonicalizedDeviceDescriptor(descriptor),
      mod
    );
  }

  /**
   * Use skipIfDeviceDoesNotHaveFeature or similar. If you really need to test
   * lack of a feature (for example tests under webgpu/api/validation/capability_checks)
   * then use UniqueFeaturesOrLimitsGPUTest
   */
  selectDeviceOrSkipTestCase(descriptor) {
    unreachable('this function should not be called in AllFeaturesMaxLimitsGPUTest');
  }

  /**
   * Use skipIfDeviceDoesNotHaveFeature or similar.
   */
  selectDeviceForQueryTypeOrSkipTestCase(types) {
    unreachable('this function should not be called in AllFeaturesMaxLimitsGPUTest');
  }

  /**
   * Use skipIfDeviceDoesNotHaveFeature or skipIf(device.limits.maxXXX < requiredXXX) etc...
   */
  selectDeviceForTextureFormatOrSkipTestCase(
  formats)
  {
    unreachable('this function should not be called in AllFeaturesMaxLimitsGPUTest');
  }

  /**
   * Use skipIfDeviceDoesNotHaveFeature or skipIf(device.limits.maxXXX < requiredXXX) etc...
   */
  selectMismatchedDeviceOrSkipTestCase(descriptor) {
    unreachable('this function should not be called in AllFeaturesMaxLimitsGPUTest');
  }
}

/**
 * Most tests should be using `AllFeaturesMaxLimitsGPUTest`. The exceptions
 * are tests specifically validating limits like those under api/validation/capability_checks/limits
 * and those tests the specifically validate certain features fail validation if not enabled
 * like those under api/validation/capability_checks/feature.
 *
 * NOTE: The goal is to go through all existing tests and remove any direct use of GPUTest.
 * For each test, choose either AllFeaturesMaxLimitsGPUTest or UniqueFeaturesOrLimitsGPUTest.
 * This way we can track progress as we go through every test using GPUTest and check it is
 * testing everything it should test.
 */
export class UniqueFeaturesOrLimitsGPUTest extends GPUTest {}

/**
 * A test that requests all features and maximum limits. This should be the default
 * test for the majority of tests, otherwise optional features will not be tested.
 * The exceptions are only tests that explicitly test the absence of a feature or
 * specific limits such as the tests under validation/capability_checks.
 *
 * As a concrete example to demonstrate the issue, texture format `rg11b10ufloat` is
 * optionally renderable and can optionally be used multisampled. Any test that tests
 * texture formats should test this format, skipping only if the feature is missing.
 * So, the default should be that the test tests `kAllTextureFormats` with the appropriate
 * filters from format_info.ts or the various helpers. This way, `rg11b10ufloat` will
 * included in the test and fail if not appropriately filtered. If instead you were
 * to use GPUTest then `rg11b10ufloat` would just be skipped as its never enabled.
 * You could enable it manually but that spreads enabling to every test instead of being
 * centralized in one place, here.
 */
export class AllFeaturesMaxLimitsGPUTest extends GPUTest {
  static MakeSharedState(
  recorder,
  params)
  {
    return new AllFeaturesMaxLimitsGPUTestSubcaseBatchState(recorder, params);
  }
}