/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { Fixture,


  SkipTestCase,
  SubcaseBatchState } from


'../common/framework/fixture.js';
import { globalTestConfig } from '../common/framework/test_config.js';
import { getGPU } from '../common/util/navigator_gpu.js';
import {
  assert,
  makeValueTestVariant,
  memcpy,
  range,



  unreachable } from
'../common/util/util.js';

import {
  getDefaultLimits,

  kQueryTypeInfo } from

'./capability_info.js';

import {
  kTextureFormatInfo,
  kEncodableTextureFormats,
  resolvePerAspectFormat,


  isCompressedTextureFormat,

  isTextureFormatUsableAsStorageFormat } from
'./format_info.js';
import { checkElementsEqual, checkElementsBetween } from './util/check_contents.js';
import { CommandBufferMaker } from './util/command_buffer_maker.js';

import { DevicePool } from './util/device_pool.js';
import { align, roundDown } from './util/math.js';
import { physicalMipSizeFromTexture, virtualMipSize } from './util/texture/base.js';
import {
  bytesInACompleteRow,
  getTextureCopyLayout,
  getTextureSubCopyLayout } from

'./util/texture/layout.js';
import { kTexelRepresentationInfo } from './util/texture/texel_data.js';
import { TexelView } from './util/texture/texel_view.js';
import {



  textureContentIsOKByT2B } from
'./util/texture/texture_ok.js';
import { createTextureFromTexelViews } from './util/texture.js';
import { reifyExtent3D, reifyOrigin3D } from './util/unions.js';

const devicePool = new DevicePool();

// MAINTENANCE_TODO: When DevicePool becomes able to provide multiple devices at once, use the
// usual one instead of a new one.
const mismatchedDevicePool = new DevicePool();

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
    return descriptor;
  }
}

export class GPUTestSubcaseBatchState extends SubcaseBatchState {
  /** Provider for default device. */

  /** Provider for mismatched device. */


  async postInit() {
    // Skip all subcases if there's no device.
    await this.acquireProvider();
  }

  async finalize() {
    await super.finalize();

    // Ensure devicePool.release is called for both providers even if one rejects.
    await Promise.all([
    this.provider?.then((x) => devicePool.release(x)),
    this.mismatchedProvider?.then((x) => devicePool.release(x))]
    );
  }

  /** @internal MAINTENANCE_TODO: Make this not visible to test code? */
  acquireProvider() {
    if (this.provider === undefined) {
      this.selectDeviceOrSkipTestCase(undefined);
    }
    assert(this.provider !== undefined);
    return this.provider;
  }

  get isCompatibility() {
    return globalTestConfig.compatibility;
  }

  getDefaultLimits() {
    return getDefaultLimits(this.isCompatibility ? 'compatibility' : 'core');
  }

  /**
   * Some tests or cases need particular feature flags or limits to be enabled.
   * Call this function with a descriptor or feature name (or `undefined`) to select a
   * GPUDevice with matching capabilities. If this isn't called, a default device is provided.
   *
   * If the request isn't supported, throws a SkipTestCase exception to skip the entire test case.
   */
  selectDeviceOrSkipTestCase(descriptor) {
    assert(this.provider === undefined, "Can't selectDeviceOrSkipTestCase() multiple times");
    this.provider = devicePool.acquire(
      this.recorder,
      initUncanonicalizedDeviceDescriptor(descriptor)
    );
    // Suppress uncaught promise rejection (we'll catch it later).
    this.provider.catch(() => {});
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
        this.skipIfTextureFormatNotSupported(format);
        features.add(kTextureFormatInfo[format].feature);
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

  /**
   * Some tests need a second device which is different from the first.
   * This requests a second device so it will be available during the test. If it is not called,
   * no second device will be available.
   *
   * If the request isn't supported, throws a SkipTestCase exception to skip the entire test case.
   */
  selectMismatchedDeviceOrSkipTestCase(descriptor) {
    assert(
      this.mismatchedProvider === undefined,
      "Can't selectMismatchedDeviceOrSkipTestCase() multiple times"
    );

    this.mismatchedProvider = mismatchedDevicePool.acquire(
      this.recorder,
      initUncanonicalizedDeviceDescriptor(descriptor)
    );
    // Suppress uncaught promise rejection (we'll catch it later).
    this.mismatchedProvider.catch(() => {});
  }

  /** Throws an exception marking the subcase as skipped. */
  skip(msg) {
    throw new SkipTestCase(msg);
  }

  /** Throws an exception making the subcase as skipped if condition is true */
  skipIf(cond, msg = '') {
    if (cond) {
      this.skip(typeof msg === 'function' ? msg() : msg);
    }
  }

  /**
   * Skips test if any format is not supported.
   */
  skipIfTextureFormatNotSupported(...formats) {
    if (this.isCompatibility) {
      for (const format of formats) {
        if (format === 'bgra8unorm-srgb') {
          this.skip(`texture format '${format} is not supported`);
        }
      }
    }
  }

  skipIfCopyTextureToTextureNotSupportedForFormat(...formats) {
    if (this.isCompatibility) {
      for (const format of formats) {
        if (format && isCompressedTextureFormat(format)) {
          this.skip(`copyTextureToTexture with ${format} is not supported`);
        }
      }
    }
  }

  skipIfTextureViewDimensionNotSupported(...dimensions) {
    if (this.isCompatibility) {
      for (const dimension of dimensions) {
        if (dimension === 'cube-array') {
          this.skip(`texture view dimension '${dimension}' is not supported`);
        }
      }
    }
  }

  skipIfTextureFormatNotUsableAsStorageTexture(...formats) {
    for (const format of formats) {
      if (format && !isTextureFormatUsableAsStorageFormat(format, this.isCompatibility)) {
        this.skip(`Texture with ${format} is not usable as a storage texture`);
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

  getDefaultLimits() {
    return getDefaultLimits(this.isCompatibility ? 'compatibility' : 'core');
  }

  getDefaultLimit(limit) {
    return this.getDefaultLimits()[limit].default;
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
      size,
      usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST
    });

    const c = this.device.createCommandEncoder();
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
   * Skips test if any format is not supported.
   */
  skipIfTextureFormatNotSupported(...formats) {
    if (this.isCompatibility) {
      for (const format of formats) {
        if (format === 'bgra8unorm-srgb') {
          this.skip(`texture format '${format} is not supported`);
        }
      }
    }
  }

  skipIfTextureViewDimensionNotSupported(...dimensions) {
    if (this.isCompatibility) {
      for (const dimension of dimensions) {
        if (dimension === 'cube-array') {
          this.skip(`texture view dimension '${dimension}' is not supported`);
        }
      }
    }
  }

  skipIfCopyTextureToTextureNotSupportedForFormat(...formats) {
    if (this.isCompatibility) {
      for (const format of formats) {
        if (format && isCompressedTextureFormat(format)) {
          this.skip(`copyTextureToTexture with ${format} is not supported`);
        }
      }
    }
  }

  skipIfTextureFormatNotUsableAsStorageTexture(...formats) {
    for (const format of formats) {
      if (format && !isTextureFormatUsableAsStorageFormat(format, this.isCompatibility)) {
        this.skip(`Texture with ${format} is not usable as a storage texture`);
      }
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

    const commandEncoder = this.device.createCommandEncoder();
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
      size: byteLength,
      usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
    });

    const commandEncoder = this.device.createCommandEncoder();
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
      size: byteLength,
      usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
    });

    const commandEncoder = this.device.createCommandEncoder();
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

    const encoder = this.device.createCommandEncoder();
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
    occlusionQuerySet



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
          const pass = this.createEncoder('render pass', { attachmentInfo });

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
            kTextureFormatInfo[fullAttachmentInfo.depthStencilFormat].depth &&
            !fullAttachmentInfo.depthReadOnly)
            {
              depthStencilAttachment.depthClearValue = 0;
              depthStencilAttachment.depthLoadOp = 'clear';
              depthStencilAttachment.depthStoreOp = 'discard';
            }
            if (
            kTextureFormatInfo[fullAttachmentInfo.depthStencilFormat].stencil &&
            !fullAttachmentInfo.stencilReadOnly)
            {
              depthStencilAttachment.stencilClearValue = 1;
              depthStencilAttachment.stencilLoadOp = 'clear';
              depthStencilAttachment.stencilStoreOp = 'discard';
            }
          }
          const passDesc = {
            colorAttachments: Array.from(fullAttachmentInfo.colorFormats, (format) =>
            format ?
            {
              view: makeAttachmentView(format),
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
      'selectMismatchedDeviceOrSkipTestCase was not called in beforeAllSubcases'
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
 * Texture expectation mixin can be applied on top of GPUTest to add texture
 * related expectation helpers.
 */

































































































































const s_deviceToResourcesMap = new WeakMap();

/**
 * Gets a (cached) pipeline to render a texture to an rgba8unorm texture
 */
function getPipelineToRenderTextureToRGB8UnormTexture(
device,
texture,
isCompatibility)
{
  if (!s_deviceToResourcesMap.has(device)) {
    s_deviceToResourcesMap.set(device, {
      pipelineByPipelineType: new Map()
    });
  }

  const { pipelineByPipelineType } = s_deviceToResourcesMap.get(device);
  const pipelineType =
  isCompatibility && texture.depthOrArrayLayers > 1 ? '2d-array' : '2d';
  if (!pipelineByPipelineType.get(pipelineType)) {
    const [textureType, layerCode] =
    pipelineType === '2d' ? ['texture_2d', ''] : ['texture_2d_array', ', uni.baseArrayLayer'];
    const module = device.createShaderModule({
      code: `
        struct VSOutput {
          @builtin(position) position: vec4f,
          @location(0) texcoord: vec2f,
        };

        struct Uniforms {
          baseArrayLayer: u32,
        };

        @vertex fn vs(
          @builtin(vertex_index) vertexIndex : u32
        ) -> VSOutput {
            let pos = array(
               vec2f(-1, -1),
               vec2f(-1,  3),
               vec2f( 3, -1),
            );

            var vsOutput: VSOutput;

            let xy = pos[vertexIndex];

            vsOutput.position = vec4f(xy, 0.0, 1.0);
            vsOutput.texcoord = xy * vec2f(0.5, -0.5) + vec2f(0.5);

            return vsOutput;
         }

         @group(0) @binding(0) var ourSampler: sampler;
         @group(0) @binding(1) var ourTexture: ${textureType}<f32>;
         @group(0) @binding(2) var<uniform> uni: Uniforms;

         @fragment fn fs(fsInput: VSOutput) -> @location(0) vec4f {
            return textureSample(ourTexture, ourSampler, fsInput.texcoord${layerCode});
         }
      `
    });
    const pipeline = device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module,
        entryPoint: 'vs'
      },
      fragment: {
        module,
        entryPoint: 'fs',
        targets: [{ format: 'rgba8unorm' }]
      }
    });
    pipelineByPipelineType.set(pipelineType, pipeline);
  }
  const pipeline = pipelineByPipelineType.get(pipelineType);
  return { pipelineType, pipeline };
}







export function TextureTestMixin(
Base)
{
  class TextureExpectations extends
  Base

  {
    /**
     * Creates a 1 mip level texture with the contents of a TexelView.
     */
    createTextureFromTexelView(
    texelView,
    desc)
    {
      return createTextureFromTexelViews(this, [texelView], desc);
    }

    createTextureFromTexelViewsMultipleMipmaps(
    texelViews,
    desc)
    {
      return createTextureFromTexelViews(this, texelViews, desc);
    }

    expectTexelViewComparisonIsOkInTexture(
    src,
    exp,
    size,
    comparisonOptions = {
      maxIntDiff: 0,
      maxDiffULPsForNormFormat: 1,
      maxDiffULPsForFloatFormat: 1
    })
    {
      this.eventualExpectOK(
        textureContentIsOKByT2B(this, src, size, { expTexelView: exp }, comparisonOptions)
      );
    }

    expectSinglePixelComparisonsAreOkInTexture(
    src,
    exp,
    comparisonOptions = {
      maxIntDiff: 0,
      maxDiffULPsForNormFormat: 1,
      maxDiffULPsForFloatFormat: 1
    })
    {
      assert(exp.length > 0, 'must specify at least one pixel comparison');
      assert(
        kEncodableTextureFormats.includes(src.texture.format),
        () => `${src.texture.format} is not an encodable format`
      );
      const lowerCorner = [src.texture.width, src.texture.height, src.texture.depthOrArrayLayers];
      const upperCorner = [0, 0, 0];
      const expMap = new Map();
      const coords = [];
      for (const e of exp) {
        const coord = reifyOrigin3D(e.coord);
        const coordKey = JSON.stringify(coord);
        coords.push(coord);

        // Compute the minimum sub-rect that encompasses all the pixel comparisons. The
        // `lowerCorner` will become the origin, and the `upperCorner` will be used to compute the
        // size.
        lowerCorner[0] = Math.min(lowerCorner[0], coord.x);
        lowerCorner[1] = Math.min(lowerCorner[1], coord.y);
        lowerCorner[2] = Math.min(lowerCorner[2], coord.z);
        upperCorner[0] = Math.max(upperCorner[0], coord.x);
        upperCorner[1] = Math.max(upperCorner[1], coord.y);
        upperCorner[2] = Math.max(upperCorner[2], coord.z);

        // Build a sparse map of the coordinates to the expected colors for the texel view.
        assert(
          !expMap.has(coordKey),
          () => `duplicate pixel expectation at coordinate (${coord.x},${coord.y},${coord.z})`
        );
        expMap.set(coordKey, e.exp);
      }
      const size = [
      upperCorner[0] - lowerCorner[0] + 1,
      upperCorner[1] - lowerCorner[1] + 1,
      upperCorner[2] - lowerCorner[2] + 1];

      let expTexelView;
      if (Symbol.iterator in exp[0].exp) {
        expTexelView = TexelView.fromTexelsAsBytes(
          src.texture.format,
          (coord) => {
            const res = expMap.get(JSON.stringify(coord));
            assert(
              res !== undefined,
              () => `invalid coordinate (${coord.x},${coord.y},${coord.z}) in sparse texel view`
            );
            return res;
          }
        );
      } else {
        expTexelView = TexelView.fromTexelsAsColors(
          src.texture.format,
          (coord) => {
            const res = expMap.get(JSON.stringify(coord));
            assert(
              res !== undefined,
              () => `invalid coordinate (${coord.x},${coord.y},${coord.z}) in sparse texel view`
            );
            return res;
          }
        );
      }
      const coordsF = function* () {
        for (const coord of coords) {
          yield coord;
        }
      }();

      this.eventualExpectOK(
        textureContentIsOKByT2B(
          this,
          { ...src, origin: reifyOrigin3D(lowerCorner) },
          size,
          { expTexelView },
          comparisonOptions,
          coordsF
        )
      );
    }

    expectTexturesToMatchByRendering(
    actualTexture,
    expectedTexture,
    mipLevel,
    origin,
    size)
    {
      // Render every layer of both textures at mipLevel to an rgba8unorm texture
      // that matches the size of the mipLevel. After each render, copy the
      // result to a buffer and expect the results from both textures to match.
      const { pipelineType, pipeline } = getPipelineToRenderTextureToRGB8UnormTexture(
        this.device,
        actualTexture,
        this.isCompatibility
      );
      const readbackPromisesPerTexturePerLayer = [actualTexture, expectedTexture].map(
        (texture, ndx) => {
          const attachmentSize = virtualMipSize('2d', [texture.width, texture.height, 1], mipLevel);
          const attachment = this.createTextureTracked({
            label: `readback${ndx}`,
            size: attachmentSize,
            format: 'rgba8unorm',
            usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
          });

          const sampler = this.device.createSampler();

          const numLayers = texture.depthOrArrayLayers;
          const readbackPromisesPerLayer = [];

          const uniformBuffer = this.createBufferTracked({
            size: 4,
            usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST
          });

          for (let layer = 0; layer < numLayers; ++layer) {
            const viewDescriptor = {
              baseMipLevel: mipLevel,
              mipLevelCount: 1,
              ...(!this.isCompatibility && {
                baseArrayLayer: layer,
                arrayLayerCount: 1
              }),
              dimension: pipelineType
            };

            const bindGroup = this.device.createBindGroup({
              layout: pipeline.getBindGroupLayout(0),
              entries: [
              { binding: 0, resource: sampler },
              {
                binding: 1,
                resource: texture.createView(viewDescriptor)
              },
              ...(pipelineType === '2d-array' ?
              [
              {
                binding: 2,
                resource: { buffer: uniformBuffer }
              }] :

              [])]

            });

            this.device.queue.writeBuffer(uniformBuffer, 0, new Uint32Array([layer]));

            const encoder = this.device.createCommandEncoder();
            const pass = encoder.beginRenderPass({
              colorAttachments: [
              {
                view: attachment.createView(),
                clearValue: [0.5, 0.5, 0.5, 0.5],
                loadOp: 'clear',
                storeOp: 'store'
              }]

            });
            pass.setPipeline(pipeline);
            pass.setBindGroup(0, bindGroup);
            pass.draw(3);
            pass.end();
            this.queue.submit([encoder.finish()]);

            const buffer = this.copyWholeTextureToNewBufferSimple(attachment, 0);

            readbackPromisesPerLayer.push(
              this.readGPUBufferRangeTyped(buffer, {
                type: Uint8Array,
                typedLength: buffer.size
              })
            );
          }
          return readbackPromisesPerLayer;
        }
      );

      this.eventualAsyncExpectation(async (niceStack) => {
        const readbacksPerTexturePerLayer = [];

        // Wait for all buffers to be ready
        for (const readbackPromises of readbackPromisesPerTexturePerLayer) {
          readbacksPerTexturePerLayer.push(await Promise.all(readbackPromises));
        }

        function arrayNotAllTheSameValue(arr, msg) {
          const first = arr[0];
          return arr.length <= 1 || arr.findIndex((v) => v !== first) >= 0 ?
          undefined :
          Error(`array is entirely ${first} so likely nothing was tested: ${msg || ''}`);
        }

        // Compare each layer of each texture as read from buffer.
        const [actualReadbacksPerLayer, expectedReadbacksPerLayer] = readbacksPerTexturePerLayer;
        for (let layer = 0; layer < actualReadbacksPerLayer.length; ++layer) {
          const actualReadback = actualReadbacksPerLayer[layer];
          const expectedReadback = expectedReadbacksPerLayer[layer];
          const sameOk =
          size.width === 0 ||
          size.height === 0 ||
          layer < origin.z ||
          layer >= origin.z + size.depthOrArrayLayers;
          this.expectOK(
            sameOk ? undefined : arrayNotAllTheSameValue(actualReadback.data, 'actualTexture')
          );
          this.expectOK(
            sameOk ? undefined : arrayNotAllTheSameValue(expectedReadback.data, 'expectedTexture')
          );
          this.expectOK(checkElementsEqual(actualReadback.data, expectedReadback.data), {
            mode: 'fail',
            niceStack
          });
          actualReadback.cleanup();
          expectedReadback.cleanup();
        }
      });
    }

    copyWholeTextureToNewBufferSimple(texture, mipLevel) {
      const { blockWidth, blockHeight, bytesPerBlock } = kTextureFormatInfo[texture.format];
      const mipSize = physicalMipSizeFromTexture(texture, mipLevel);
      assert(bytesPerBlock !== undefined);

      const blocksPerRow = mipSize[0] / blockWidth;
      const blocksPerColumn = mipSize[1] / blockHeight;

      assert(blocksPerRow % 1 === 0);
      assert(blocksPerColumn % 1 === 0);

      const bytesPerRow = align(blocksPerRow * bytesPerBlock, 256);
      const byteLength = bytesPerRow * blocksPerColumn * mipSize[2];

      return this.copyWholeTextureToNewBuffer(
        { texture, mipLevel },
        {
          bytesPerBlock,
          bytesPerRow,
          rowsPerImage: blocksPerColumn,
          byteLength
        }
      );
    }

    copyWholeTextureToNewBuffer(
    { texture, mipLevel },
    resultDataLayout)





    {
      const { byteLength, bytesPerRow, rowsPerImage } = resultDataLayout;
      const buffer = this.createBufferTracked({
        size: align(byteLength, 4), // this is necessary because we need to copy and map data from this buffer
        usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
      });

      const mipSize = physicalMipSizeFromTexture(texture, mipLevel || 0);
      const encoder = this.device.createCommandEncoder();
      encoder.copyTextureToBuffer(
        { texture, mipLevel },
        { buffer, bytesPerRow, rowsPerImage },
        mipSize
      );
      this.device.queue.submit([encoder.finish()]);

      return buffer;
    }

    updateLinearTextureDataSubBox(
    format,
    copySize,
    copyParams)



    {
      const { src, dest } = copyParams;
      const rowLength = bytesInACompleteRow(copySize.width, format);
      for (const texel of this.iterateBlockRows(copySize, format)) {
        const srcOffsetElements = this.getTexelOffsetInBytes(
          src.dataLayout,
          format,
          texel,
          src.origin
        );
        const dstOffsetElements = this.getTexelOffsetInBytes(
          dest.dataLayout,
          format,
          texel,
          dest.origin
        );
        memcpy(
          { src: src.data, start: srcOffsetElements, length: rowLength },
          { dst: dest.data, start: dstOffsetElements }
        );
      }
    }

    /** Offset for a particular texel in the linear texture data */
    getTexelOffsetInBytes(
    textureDataLayout,
    format,
    texel,
    origin = { x: 0, y: 0, z: 0 })
    {
      const { offset, bytesPerRow, rowsPerImage } = textureDataLayout;
      const info = kTextureFormatInfo[format];

      assert(texel.x % info.blockWidth === 0);
      assert(texel.y % info.blockHeight === 0);
      assert(origin.x % info.blockWidth === 0);
      assert(origin.y % info.blockHeight === 0);

      const bytesPerImage = rowsPerImage * bytesPerRow;

      return (
        offset +
        (texel.z + origin.z) * bytesPerImage +
        (texel.y + origin.y) / info.blockHeight * bytesPerRow +
        (texel.x + origin.x) / info.blockWidth * info.color.bytes);

    }

    *iterateBlockRows(
    size,
    format)
    {
      if (size.width === 0 || size.height === 0 || size.depthOrArrayLayers === 0) {
        // do not iterate anything for an empty region
        return;
      }
      const info = kTextureFormatInfo[format];
      assert(size.height % info.blockHeight === 0);
      // Note: it's important that the order is in increasing memory address order.
      for (let z = 0; z < size.depthOrArrayLayers; ++z) {
        for (let y = 0; y < size.height; y += info.blockHeight) {
          yield {
            x: 0,
            y,
            z
          };
        }
      }
    }
  }

  return TextureExpectations;
}