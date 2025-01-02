/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
TODO: Test more corner case values for Float16 / Float32 (INF, NaN, ...) and reduce the
float tolerance.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import {
  assert,
  filterUniqueValueTestVariants,
  makeValueTestVariant,
  memcpy,
  unreachable } from
'../../../../common/util/util.js';
import {
  kPerStageBindingLimits,
  kVertexFormatInfo,
  kVertexFormats } from
'../../../capability_info.js';
import { GPUTest } from '../../../gpu_test.js';
import { float32ToFloat16Bits, normalizedIntegerAsFloat } from '../../../util/conversion.js';
import { align, clamp } from '../../../util/math.js';

// These types mirror the structure of GPUVertexBufferLayout but allow defining the extra
// dictionary members at the GPUVertexBufferLayout and GPUVertexAttribute level. The are used
// like so:
//
//   VertexState<{arrayStride: number}, {format: VertexFormat}>
//   VertexBuffer<{arrayStride: number}, {format: VertexFormat}>
//   VertexAttrib<{format: VertexFormat}>












function mapBufferAttribs(
buffer,
f)
{
  const newAttributes = [];
  for (const a of buffer.attributes) {
    newAttributes.push({
      shaderLocation: a.shaderLocation,
      ...f(buffer, a)
    });
  }

  return { ...buffer, attributes: newAttributes };
}

function mapStateAttribs(
buffers,
f)
{
  return buffers.map((b) => mapBufferAttribs(b, f));
}

function makeRgb10a2(rgba) {
  const [r, g, b, a] = rgba;
  assert((r & 0x3ff) === r);
  assert((g & 0x3ff) === g);
  assert((b & 0x3ff) === b);
  assert((a & 0x3) === a);
  return r | g << 10 | b << 20 | a << 30;
}

function normalizeRgb10a2(rgba, index) {
  const normalizationFactor = index % 4 === 3 ? 3 : 1023;
  return rgba / normalizationFactor;
}













class VertexStateTest extends GPUTest {
  // Generate for VS + FS (entrypoints vsMain / fsMain) that for each attribute will check that its
  // value corresponds to what's expected (as provided by a uniform buffer per attribute) and then
  // renders each vertex at position (vertexIndex, instanceindex) with either 1 (success) or
  // a negative number corresponding to the check number (in case you need to debug a failure).
  makeTestWGSL(
  buffers,








  vertexCount,
  instanceCount)
  {
    // In the base WebGPU spec maxVertexAttributes is larger than maxUniformBufferPerStage. We'll
    // use a combination of uniform and storage buffers to cover all possible attributes. This
    // happens to work because maxUniformBuffer + maxStorageBuffer = 12 + 8 = 20 which is larger
    // than maxVertexAttributes = 16.
    // However this might not work in the future for implementations that allow even more vertex
    // attributes so there will need to be larger changes when that happens.
    const maxUniformBuffers = this.getDefaultLimit(kPerStageBindingLimits['uniformBuf'].maxLimit);
    assert(
      maxUniformBuffers + this.getDefaultLimit(kPerStageBindingLimits['storageBuf'].maxLimit) >=
      this.device.limits.maxVertexAttributes
    );

    let vsInputs = '';
    let vsChecks = '';
    let vsBindings = '';

    for (const b of buffers) {
      for (const a of b.attributes) {
        const format = kVertexFormatInfo[a.format];
        const shaderComponentCount = a.shaderComponentCount ?? format.componentCount;
        const i = a.shaderLocation;

        // shaderType is either a scalar type like f32 or a vecN<scalarType>
        let shaderType = a.shaderBaseType;
        if (shaderComponentCount !== 1) {
          shaderType = `vec${shaderComponentCount}<${shaderType}>`;
        }

        let maxCount = `${vertexCount}`;
        let indexBuiltin = `input.vertexIndex`;
        if (b.stepMode === 'instance') {
          maxCount = `${instanceCount}`;
          indexBuiltin = `input.instanceIndex`;
        }

        // Start using storage buffers when we run out of uniform buffers.
        let storageType = 'uniform';
        if (i >= maxUniformBuffers) {
          storageType = 'storage, read';
        }

        vsInputs += `  @location(${i}) attrib${i} : ${shaderType},\n`;
        vsBindings += `struct S${i} { data : array<vec4<${a.shaderBaseType}>, ${maxCount}> };\n`;
        vsBindings += `@group(0) @binding(${i}) var<${storageType}> providedData${i} : S${i};\n`;

        // Generate the all the checks for the attributes.
        for (let component = 0; component < shaderComponentCount; component++) {
          // Components are filled with (0, 0, 0, 1) if they aren't provided data from the pipeline.
          if (component >= format.componentCount) {
            const expected = component === 3 ? '1' : '0';
            vsChecks += `  check(input.attrib${i}[${component}] == ${a.shaderBaseType}(${expected}));\n`;
            continue;
          }

          // Check each component individually, with special handling of tolerance for floats.
          const attribComponent =
          shaderComponentCount === 1 ? `input.attrib${i}` : `input.attrib${i}[${component}]`;
          const providedData = `providedData${i}.data[${indexBuiltin}][${component}]`;
          if (format.type === 'uint' || format.type === 'sint') {
            vsChecks += `  check(${attribComponent} == ${providedData});\n`;
          } else {
            vsChecks += `  check(floatsSimilar(${attribComponent}, ${providedData}, f32(${
            a.floatTolerance ?? 0
            })));\n`;
          }
        }
      }
    }

    return `
struct Inputs {
${vsInputs}
  @builtin(vertex_index) vertexIndex: u32,
  @builtin(instance_index) instanceIndex: u32,
};

${vsBindings}

var<private> vsResult : i32 = 1;
var<private> checkIndex : i32 = 0;
fn check(success : bool) {
  if (!success) {
    vsResult = -checkIndex;
  }
  checkIndex = checkIndex + 1;
}

fn floatsSimilar(a : f32, b : f32, tolerance : f32) -> bool {
  // Note: -0.0 and 0.0 have different bit patterns, but compare as equal.
  return abs(a - b) < tolerance;
}

fn doTest(input : Inputs) {
${vsChecks}
}

struct VSOutputs {
  @location(0) @interpolate(flat, either) result : i32,
  @builtin(position) position : vec4<f32>,
};

@vertex fn vsMain(input : Inputs) -> VSOutputs {
  doTest(input);

  // Place that point at pixel (vertexIndex, instanceIndex) in a framebuffer of size
  // (vertexCount , instanceCount).
  var output : VSOutputs;
  output.position = vec4<f32>(
    ((f32(input.vertexIndex) + 0.5) / ${vertexCount}.0 * 2.0) - 1.0,
    ((f32(input.instanceIndex) + 0.5) / ${instanceCount}.0 * 2.0) - 1.0,
    0.0, 1.0
  );
  output.result = vsResult;
  return output;
}

@fragment fn fsMain(@location(0) @interpolate(flat, either) result : i32)
  -> @location(0) i32 {
  return result;
}
    `;
  }

  makeTestPipeline(
  buffers,









  vertexCount,
  instanceCount)
  {
    const module = this.device.createShaderModule({
      code: this.makeTestWGSL(buffers, vertexCount, instanceCount)
    });

    const bufferLayouts = [];
    for (const b of buffers) {
      bufferLayouts[b.slot] = b;
    }

    return this.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module,
        entryPoint: 'vsMain',
        buffers: bufferLayouts
      },
      primitive: {
        topology: 'point-list'
      },
      fragment: {
        module,
        entryPoint: 'fsMain',
        targets: [
        {
          format: 'r32sint'
        }]

      }
    });
  }

  // Runs the render pass drawing points in a vertexCount*instanceCount rectangle, then check each
  // of produced a value of 1 which means that the tests in the shader passed.
  submitRenderPass(
  pipeline,
  buffers,
  expectedData,
  vertexCount,
  instanceCount)
  {
    const testTexture = this.createTextureTracked({
      format: 'r32sint',
      size: [vertexCount, instanceCount],
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
    });

    const encoder = this.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: testTexture.createView(),
        clearValue: [0, 0, 0, 0],
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });

    pass.setPipeline(pipeline);
    pass.setBindGroup(0, expectedData);
    for (const buffer of buffers) {
      pass.setVertexBuffer(buffer.slot, buffer.buffer, buffer.vbOffset ?? 0);
    }
    pass.draw(vertexCount, instanceCount);
    pass.end();

    this.device.queue.submit([encoder.finish()]);

    this.expectSingleColor(testTexture, 'r32sint', {
      size: [vertexCount, instanceCount, 1],
      exp: { R: 1 }
    });
  }

  // Generate TestData for the format with interesting test values.
  // MAINTENANCE_TODO cache the result on the fixture?
  // Note that the test data always starts with an interesting value, so that using the first
  // test value in a test is still meaningful.
  generateTestData(format) {
    const formatInfo = kVertexFormatInfo[format];
    const bitSize =
    formatInfo.bytesPerComponent === 'packed' ? 0 : formatInfo.bytesPerComponent * 8;

    switch (formatInfo.type) {
      case 'float':{
          // -0.0 and +0.0 have different bit patterns, but compare as equal.
          const data = [42.42, 0.0, -0.0, 1.0, -1.0, 1000, -18.7, 25.17];
          const expectedData = new Float32Array(data).buffer;
          const vertexData =
          bitSize === 32 ?
          expectedData :
          bitSize === 16 ?
          new Uint16Array(data.map(float32ToFloat16Bits)).buffer :
          unreachable();

          return {
            shaderBaseType: 'f32',
            testComponentCount: data.length,
            expectedData,
            vertexData,
            floatTolerance: 0.05
          };
        }

      case 'sint':{

          const data = [
          42,
          0, 1, 2, 3, 4, 5,
          -1, -2, -3, -4, -5,
          Math.pow(2, bitSize - 2),
          Math.pow(2, bitSize - 1) - 1, // max value
          -Math.pow(2, bitSize - 2),
          -Math.pow(2, bitSize - 1) // min value
          ];
          const expectedData = new Int32Array(data).buffer;
          const vertexData =
          bitSize === 32 ?
          expectedData :
          bitSize === 16 ?
          new Int16Array(data).buffer :
          new Int8Array(data).buffer;

          return {
            shaderBaseType: 'i32',
            testComponentCount: data.length,
            expectedData,
            vertexData
          };
        }

      case 'uint':{

          const data = [
          42,
          0, 1, 2, 3, 4, 5,
          Math.pow(2, bitSize - 1),
          Math.pow(2, bitSize) - 1 // max value
          ];
          const expectedData = new Uint32Array(data).buffer;
          const vertexData =
          bitSize === 32 ?
          expectedData :
          bitSize === 16 ?
          new Uint16Array(data).buffer :
          new Uint8Array(data).buffer;

          return {
            shaderBaseType: 'u32',
            testComponentCount: data.length,
            expectedData,
            vertexData
          };
        }

      case 'snorm':{

          const data = [
          42,
          0, 1, 2, 3, 4, 5,
          -1, -2, -3, -4, -5,
          Math.pow(2, bitSize - 2),
          Math.pow(2, bitSize - 1) - 1, // max value
          -Math.pow(2, bitSize - 2),
          -Math.pow(2, bitSize - 1) // min value
          ];
          const vertexData =
          bitSize === 16 ?
          new Int16Array(data).buffer :
          bitSize === 8 ?
          new Int8Array(data).buffer :
          unreachable();

          return {
            shaderBaseType: 'f32',
            testComponentCount: data.length,
            expectedData: new Float32Array(data.map((v) => normalizedIntegerAsFloat(v, bitSize, true))).
            buffer,
            vertexData,
            floatTolerance: 0.1 * normalizedIntegerAsFloat(1, bitSize, true)
          };
        }

      case 'unorm':{
          if (formatInfo.bytesPerComponent === 'packed') {
            assert(format === 'unorm10-10-10-2'); // This is the only packed format for now.
            assert(bitSize === 0);


            const data = [
            [0, 0, 0, 0],
            [1023, 1023, 1023, 3],
            [243, 567, 765, 2]];

            const vertexData = new Uint32Array(data.map(makeRgb10a2)).buffer;
            const expectedData = new Float32Array(data.flat().map(normalizeRgb10a2)).buffer;

            return {
              shaderBaseType: 'f32',
              testComponentCount: data.flat().length,
              expectedData,
              vertexData,
              floatTolerance: 0.1 / 1023
            };
          }


          const data = [
          42,
          0, 1, 2, 3, 4, 5,
          Math.pow(2, bitSize - 1),
          Math.pow(2, bitSize) - 1 // max value
          ];
          const vertexData =
          bitSize === 16 ?
          new Uint16Array(data).buffer :
          bitSize === 8 ?
          new Uint8Array(data).buffer :
          unreachable();

          return {
            shaderBaseType: 'f32',
            testComponentCount: data.length,
            expectedData: new Float32Array(data.map((v) => normalizedIntegerAsFloat(v, bitSize, false))).
            buffer,
            vertexData,
            floatTolerance: 0.1 * normalizedIntegerAsFloat(1, bitSize, false)
          };
        }
    }
  }

  // The TestData generated for a format might not contain enough data for all the vertices we are
  // going to draw, so we expand them by adding additional copies of the vertexData as needed.
  // expectedData is a bit different because it also needs to be unpacked to have `componentCount`
  // components every 4 components (because the shader uses vec4 for the expected data).
  expandTestData(data, maxCount, componentCount) {
    const vertexComponentSize = data.vertexData.byteLength / data.testComponentCount;
    const expectedComponentSize = data.expectedData.byteLength / data.testComponentCount;

    const expandedVertexData = new Uint8Array(maxCount * componentCount * vertexComponentSize);
    const expandedExpectedData = new Uint8Array(4 * maxCount * expectedComponentSize);

    for (let index = 0; index < maxCount; index++) {
      for (let component = 0; component < componentCount; component++) {
        // If only we had some builtin JS memcpy function between ArrayBuffers...
        const targetVertexOffset = (index * componentCount + component) * vertexComponentSize;
        const sourceVertexOffset = targetVertexOffset % data.vertexData.byteLength;
        memcpy(
          { src: data.vertexData, start: sourceVertexOffset, length: vertexComponentSize },
          { dst: expandedVertexData, start: targetVertexOffset }
        );

        const targetExpectedOffset = (index * 4 + component) * expectedComponentSize;
        const sourceExpectedOffset =
        (index * componentCount + component) * expectedComponentSize %
        data.expectedData.byteLength;
        memcpy(
          { src: data.expectedData, start: sourceExpectedOffset, length: expectedComponentSize },
          { dst: expandedExpectedData, start: targetExpectedOffset }
        );
      }
    }

    return {
      shaderBaseType: data.shaderBaseType,
      testComponentCount: maxCount * componentCount,
      floatTolerance: data.floatTolerance,
      expectedData: expandedExpectedData.buffer,
      vertexData: expandedVertexData.buffer
    };
  }

  // Copies `size` bytes from `source` to `target` starting at `offset` each `targetStride`.
  // (the data in `source` is assumed packed)
  interleaveVertexDataInto(
  target,
  src,
  { targetStride, offset, size })
  {
    const dst = new Uint8Array(target);
    for (
    let srcStart = 0, dstStart = offset;
    srcStart < src.byteLength;
    srcStart += size, dstStart += targetStride)
    {
      memcpy({ src, start: srcStart, length: size }, { dst, start: dstStart });
    }
  }

  createTestAndPipelineData(
  state,
  vertexCount,
  instanceCount)
  {
    // Gather the test data and some additional test state for attribs.
    return mapStateAttribs(state, (buffer, attrib) => {
      const maxCount = buffer.stepMode === 'instance' ? instanceCount : vertexCount;
      const formatInfo = kVertexFormatInfo[attrib.format];

      let testData = this.generateTestData(attrib.format);
      testData = this.expandTestData(testData, maxCount, formatInfo.componentCount);

      return {
        ...testData,
        ...attrib
      };
    });
  }

  createExpectedBG(state, pipeline) {
    // Create the bindgroups from that test data
    const bgEntries = [];

    for (const buffer of state) {
      for (const attrib of buffer.attributes) {
        const expectedDataBuffer = this.makeBufferWithContents(
          new Uint8Array(attrib.expectedData),
          GPUBufferUsage.UNIFORM | GPUBufferUsage.STORAGE
        );
        bgEntries.push({
          binding: attrib.shaderLocation,
          resource: { buffer: expectedDataBuffer }
        });
      }
    }

    return this.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: bgEntries
    });
  }

  createVertexBuffers(
  state,
  vertexCount,
  instanceCount)
  {
    // Create the vertex buffers
    const vertexBuffers = [];

    for (const buffer of state) {
      const maxCount = buffer.stepMode === 'instance' ? instanceCount : vertexCount;

      // Fill the vertex data with garbage so that we don't get `0` (which could be a test value)
      // if the vertex shader loads the vertex data incorrectly.
      const vertexData = new ArrayBuffer(
        align(buffer.arrayStride * maxCount + (buffer.vbOffset ?? 0), 4)
      );
      new Uint8Array(vertexData).fill(0xc4);

      for (const attrib of buffer.attributes) {
        const formatInfo = kVertexFormatInfo[attrib.format];
        this.interleaveVertexDataInto(vertexData, attrib.vertexData, {
          targetStride: buffer.arrayStride,
          offset: (buffer.vbOffset ?? 0) + attrib.offset,
          size: formatInfo.byteSize
        });
      }

      vertexBuffers.push({
        slot: buffer.slot,
        buffer: this.makeBufferWithContents(new Uint8Array(vertexData), GPUBufferUsage.VERTEX),
        vbOffset: buffer.vbOffset,
        attributes: []
      });
    }

    return vertexBuffers;
  }

  runTest(
  buffers,
  // Default to using 20 vertices and 20 instances so that we cover each of the test data at least
  // once (at the time of writing the largest testData has 16 values).
  vertexCount = 20,
  instanceCount = 20)
  {
    const testData = this.createTestAndPipelineData(buffers, vertexCount, instanceCount);
    const pipeline = this.makeTestPipeline(testData, vertexCount, instanceCount);
    const expectedDataBG = this.createExpectedBG(testData, pipeline);
    const vertexBuffers = this.createVertexBuffers(testData, vertexCount, instanceCount);
    this.submitRenderPass(pipeline, vertexBuffers, expectedDataBG, vertexCount, instanceCount);
  }
}

export const g = makeTestGroup(VertexStateTest);

g.test('vertex_format_to_shader_format_conversion').
desc(
  `Test that the raw data passed in vertex buffers is correctly converted to the input type in the shader. Test for:
  - all formats
  - 1 to 4 components in the shader's input type (unused components are filled with 0 and except the 4th with 1)
  - various locations
  - various slots`
).
params((u) =>
u //
.combine('format', kVertexFormats).
combine('shaderComponentCount', [1, 2, 3, 4]).
beginSubcases().
combine('slotVariant', [
{ mult: 0, add: 0 },
{ mult: 0, add: 1 },
{ mult: 1, add: -1 }]
).
combine('shaderLocationVariant', [
{ mult: 0, add: 0 },
{ mult: 0, add: 1 },
{ mult: 1, add: -1 }]
)
).
fn((t) => {
  const { format, shaderComponentCount, slotVariant, shaderLocationVariant } = t.params;
  const slot = t.makeLimitVariant('maxVertexBuffers', slotVariant);
  const shaderLocation = t.makeLimitVariant('maxVertexAttributes', shaderLocationVariant);
  t.runTest([
  {
    slot,
    arrayStride: 16,
    stepMode: 'vertex',
    attributes: [
    {
      shaderLocation,
      format,
      offset: 0,
      shaderComponentCount
    }]

  }]
  );
});

g.test('setVertexBuffer_offset_and_attribute_offset').
desc(
  `Test that the vertex buffer offset and attribute offset in the vertex state are applied correctly. Test for:
  - all formats
  - various setVertexBuffer offsets
  - various attribute offsets in a fixed arrayStride`
).
params((u) =>
u //
.combine('format', kVertexFormats).
beginSubcases().
combine('vbOffset', [0, 4, 400, 1004]).
combine('arrayStride', [128]).
expand('offset', (p) => {
  const formatInfo = kVertexFormatInfo[p.format];
  const formatSize = formatInfo.byteSize;
  return new Set([
  0,
  4,
  8,
  formatSize,
  formatSize * 2,
  p.arrayStride / 2,
  p.arrayStride - formatSize - 4,
  p.arrayStride - formatSize - 8,
  p.arrayStride - formatSize - formatSize,
  p.arrayStride - formatSize - formatSize * 2,
  p.arrayStride - formatSize]
  );
})
).
fn((t) => {
  const { format, vbOffset, arrayStride, offset } = t.params;
  t.runTest([
  {
    slot: 0,
    arrayStride,
    stepMode: 'vertex',
    vbOffset,
    attributes: [
    {
      shaderLocation: 0,
      format,
      offset
    }]

  }]
  );
});

g.test('non_zero_array_stride_and_attribute_offset').
desc(
  `Test that the array stride and attribute offset in the vertex state are applied correctly. Test for:
  - all formats
  - various array strides
  - various attribute offsets in a fixed arrayStride`
).
params((u) =>
u //
.combine('format', kVertexFormats).
beginSubcases().
expand('arrayStrideVariant', (p) => {
  const formatInfo = kVertexFormatInfo[p.format];
  const formatSize = formatInfo.byteSize;

  return [
  { mult: 0, add: align(formatSize, 4) },
  { mult: 0, add: align(formatSize, 4) + 4 },
  { mult: 1, add: 0 }];

}).
expand('offsetVariant', function* (p) {
  const formatInfo = kVertexFormatInfo[p.format];
  const formatSize = formatInfo.byteSize;
  yield { mult: 0, add: 0 };
  yield { mult: 0, add: 4 };
  if (formatSize !== 4) yield { mult: 0, add: formatSize };
  yield { mult: 0.5, add: 0 };
  yield { mult: 1, add: -formatSize * 2 };
  if (formatSize !== 4) yield { mult: 1, add: -formatSize - 4 };
  yield { mult: 1, add: -formatSize };
})
).
fn((t) => {
  const { format, arrayStrideVariant, offsetVariant } = t.params;
  const arrayStride = t.makeLimitVariant('maxVertexBufferArrayStride', arrayStrideVariant);
  const formatInfo = kVertexFormatInfo[format];
  const formatSize = formatInfo.byteSize;
  const offset = clamp(makeValueTestVariant(arrayStride, offsetVariant), {
    min: 0,
    max: arrayStride - formatSize
  });

  t.runTest([
  {
    slot: 0,
    arrayStride,
    stepMode: 'vertex',
    attributes: [
    {
      shaderLocation: 0,
      format,
      offset
    }]

  }]
  );
});

g.test('buffers_with_varying_step_mode').
desc(
  `Test buffers with varying step modes in the same vertex state.
  - Various combination of step modes`
).
paramsSubcasesOnly((u) =>
u //
.combine('stepModes', [
['instance'],
['vertex', 'vertex', 'instance'],
['instance', 'vertex', 'instance'],
['vertex', 'instance', 'vertex', 'vertex']]
)
).
fn((t) => {
  const { stepModes } = t.params;
  const state = stepModes.map((stepMode, i) => ({
    slot: i,
    arrayStride: 4,
    stepMode,
    attributes: [
    {
      shaderLocation: i,
      format: 'float32',
      offset: 0
    }]

  }));
  t.runTest(state);
});

g.test('vertex_buffer_used_multiple_times_overlapped').
desc(
  `Test using the same vertex buffer in for multiple "vertex buffers", with data from each buffer overlapping.
  - For each vertex format.
  - For various numbers of vertex buffers [2, 3, max]`
).
params((u) =>
u //
.combine('format', kVertexFormats).
beginSubcases().
combine('vbCountVariant', [
{ mult: 0, add: 2 },
{ mult: 0, add: 3 },
{ mult: 1, add: 0 }]
).
combine('additionalVBOffset', [0, 4, 120])
).
fn((t) => {
  const { format, vbCountVariant, additionalVBOffset } = t.params;
  const vbCount = t.makeLimitVariant('maxVertexBuffers', vbCountVariant);
  const kVertexCount = 20;
  const kInstanceCount = 1;
  const formatInfo = kVertexFormatInfo[format];
  const formatByteSize = formatInfo.byteSize;
  // We need to align so the offset for non-0 setVertexBuffer don't fail validation.
  const alignedFormatByteSize = align(formatByteSize, 4);

  // In this test we want to test using the same vertex buffer for multiple different attributes.
  // For example if vbCount is 3, we will create a vertex buffer containing the following data:
  //    a0, a1, a2, a3, ..., a<baseDataVertexCount>
  // We also create the expected data for the vertex fetching from that buffer so we can modify it
  // below.
  const baseDataVertexCount = kVertexCount + vbCount - 1;
  const baseData = t.createTestAndPipelineData(
    [
    {
      slot: 0,
      arrayStride: alignedFormatByteSize,
      stepMode: 'vertex',
      vbOffset: additionalVBOffset,
      attributes: [{ shaderLocation: 0, format, offset: 0 }]
    }],

    baseDataVertexCount,
    kInstanceCount
  );
  const vertexBuffer = t.createVertexBuffers(baseData, baseDataVertexCount, kInstanceCount)[0].
  buffer;

  // We are going to bind the vertex buffer multiple times, each time at a different offset that's
  // a multiple of the data size. So what should be fetched by the vertex shader is:
  //    - attrib0: a0, a1, ..., a19
  //    - attrib1: a1, a2, ..., a20
  //    - attrib2: a2, a3, ..., a21
  //    etc.
  // We re-create the test data by:
  //   1) creating multiple "vertex buffers" that all point at the GPUBuffer above but at
  //      different offsets.
  //   2) selecting what parts of the expectedData each attribute will see in the expectedData for
  //      the full vertex buffer.
  const baseTestData = baseData[0].attributes[0];
  assert(baseTestData.testComponentCount === formatInfo.componentCount * baseDataVertexCount);
  const expectedDataBytesPerVertex = baseTestData.expectedData.byteLength / baseDataVertexCount;

  const testData = [];
  const vertexBuffers = [];
  for (let i = 0; i < vbCount; i++) {
    vertexBuffers.push({
      buffer: vertexBuffer,
      slot: i,
      vbOffset: additionalVBOffset + i * alignedFormatByteSize,
      attributes: []
    });

    testData.push({
      slot: i,
      arrayStride: alignedFormatByteSize,
      stepMode: 'vertex',
      attributes: [
      {
        shaderLocation: i,
        format,
        offset: 0,

        shaderBaseType: baseTestData.shaderBaseType,
        floatTolerance: baseTestData.floatTolerance,
        // Select vertices [i, i + kVertexCount]
        testComponentCount: kVertexCount * formatInfo.componentCount,
        expectedData: baseTestData.expectedData.slice(
          expectedDataBytesPerVertex * i,
          expectedDataBytesPerVertex * (kVertexCount + i)
        ),
        vertexData: new ArrayBuffer(0)
      }]

    });
  }

  // Run the test with the modified test data.
  const pipeline = t.makeTestPipeline(testData, kVertexCount, kInstanceCount);
  const expectedDataBG = t.createExpectedBG(testData, pipeline);
  t.submitRenderPass(pipeline, vertexBuffers, expectedDataBG, kVertexCount, kInstanceCount);
});

g.test('vertex_buffer_used_multiple_times_interleaved').
desc(
  `Test using the same vertex buffer in for multiple "vertex buffers", with data from each buffer interleaved.
  - For each vertex format.
  - For various numbers of vertex buffers [2, 3, max]`
).
params((u) =>
u //
.combine('format', kVertexFormats).
beginSubcases().
combine('vbCountVariant', [
{ mult: 0, add: 2 },
{ mult: 0, add: 3 },
{ mult: 1, add: 0 }]
).
combine('additionalVBOffset', [0, 4, 120])
).
fn((t) => {
  const { format, vbCountVariant, additionalVBOffset } = t.params;
  const vbCount = t.makeLimitVariant('maxVertexBuffers', vbCountVariant);
  const kVertexCount = 20;
  const kInstanceCount = 1;
  const formatInfo = kVertexFormatInfo[format];
  const formatByteSize = formatInfo.byteSize;
  // We need to align so the offset for non-0 setVertexBuffer don't fail validation.
  const alignedFormatByteSize = align(formatByteSize, 4);

  // Create data for a single vertex buffer with many attributes, that will be split between
  // many vertex buffers set at different offsets.

  // In this test we want to test using the same vertex buffer for multiple different attributes.
  // For example if vbCount is 3, we will create a vertex buffer containing the following data:
  //    a0, a0, a0, a1, a1, a1, ...
  // To do that we create a single vertex buffer with `vbCount` attributes that all have the same
  // format.
  const attribs = [];
  for (let i = 0; i < vbCount; i++) {
    attribs.push({ format, offset: i * alignedFormatByteSize, shaderLocation: i });
  }
  const baseData = t.createTestAndPipelineData(
    [
    {
      slot: 0,
      arrayStride: alignedFormatByteSize * vbCount,
      stepMode: 'vertex',
      vbOffset: additionalVBOffset,
      attributes: attribs
    }],

    // Request one vertex more than what we need so we have an extra full stride. Otherwise WebGPU
    // validation of vertex being in bounds will fail for all vertex buffers at an offset that's
    // not 0 (since their last stride will go beyond the data for vertex kVertexCount -1).
    kVertexCount + 1,
    kInstanceCount
  );
  const vertexBuffer = t.createVertexBuffers(baseData, kVertexCount + 1, kInstanceCount)[0].
  buffer;

  // Then we recreate test data by:
  //   1) creating multiple "vertex buffers" that all point at the GPUBuffer above but at
  //      different offsets.
  //   2) have multiple vertex buffer, each with one attributes that will expect a0, a1, ...
  const testData = [];
  const vertexBuffers = [];
  for (let i = 0; i < vbCount; i++) {
    vertexBuffers.push({
      slot: i,
      buffer: vertexBuffer,
      vbOffset: additionalVBOffset + i * alignedFormatByteSize,
      attributes: []
    });
    testData.push({
      ...baseData[0],
      slot: i,
      attributes: [{ ...baseData[0].attributes[i], offset: 0 }]
    });
  }

  // Run the test with the modified test data.
  const pipeline = t.makeTestPipeline(testData, kVertexCount, kInstanceCount);
  const expectedDataBG = t.createExpectedBG(testData, pipeline);
  t.submitRenderPass(pipeline, vertexBuffers, expectedDataBG, kVertexCount, kInstanceCount);
});

g.test('max_buffers_and_attribs').
desc(
  `Test a vertex state that loads as many attributes and buffers as possible.
  - For each format.
  `
).
params((u) => u.combine('format', kVertexFormats)).
fn((t) => {
  const { format } = t.params;
  // In compat mode, @builtin(vertex_index) and @builtin(instance_index) each take an attribute
  const maxVertexBuffers = t.device.limits.maxVertexBuffers;
  const deviceMaxVertexAttributes = t.device.limits.maxVertexAttributes;
  const maxVertexAttributes = deviceMaxVertexAttributes - (t.isCompatibility ? 2 : 0);
  const attributesPerBuffer = Math.ceil(maxVertexAttributes / maxVertexBuffers);
  let attributesEmitted = 0;

  const state = [];
  for (let i = 0; i < maxVertexBuffers; i++) {
    const attributes = [];
    for (let j = 0; j < attributesPerBuffer && attributesEmitted < maxVertexAttributes; j++) {
      attributes.push({ format, offset: 0, shaderLocation: attributesEmitted });
      attributesEmitted++;
    }
    state.push({
      slot: i,
      stepMode: 'vertex',
      arrayStride: 32,
      attributes
    });
  }
  t.runTest(state);
});

g.test('array_stride_zero').
desc(
  `Test that arrayStride 0 correctly uses the same data for all vertex/instances, while another test vertex buffer with arrayStride != 0 gets different data.
  - Test for all formats
  - Test for both step modes`
).
params((u) =>
u //
.combine('format', kVertexFormats).
beginSubcases().
combine('stepMode', ['vertex', 'instance']).
expand('offsetVariant', (p) => {
  const formatInfo = kVertexFormatInfo[p.format];
  const formatSize = formatInfo.byteSize;
  return filterUniqueValueTestVariants([
  { mult: 0, add: 0 },
  { mult: 0, add: 4 },
  { mult: 0, add: 8 },
  { mult: 0, add: formatSize },
  { mult: 0, add: formatSize * 2 },
  { mult: 0.5, add: 0 },
  { mult: 1, add: -formatSize - 4 },
  { mult: 1, add: -formatSize - 8 },
  { mult: 1, add: -formatSize },
  { mult: 1, add: -formatSize * 2 }]
  );
})
).
fn((t) => {
  const { format, stepMode, offsetVariant } = t.params;
  const offset = t.makeLimitVariant('maxVertexBufferArrayStride', offsetVariant);
  const kCount = 10;

  // Create the stride 0 part of the test, first by faking a single vertex being drawn and
  // then expanding the data to cover kCount vertex / instances
  const stride0TestData = t.createTestAndPipelineData(
    [
    {
      slot: 0,
      arrayStride: 2048,
      stepMode,
      vbOffset: offset, // used to push data in the vertex buffer
      attributes: [{ format, offset: 0, shaderLocation: 0 }]
    }],

    1,
    1
  )[0];
  const stride0VertexBuffer = t.createVertexBuffers([stride0TestData], kCount, kCount)[0];

  // Expand the stride0 test data to have kCount values for expectedData.
  const originalData = stride0TestData.attributes[0].expectedData;
  const expandedData = new ArrayBuffer(kCount * originalData.byteLength);
  for (let i = 0; i < kCount; i++) {
    new Uint8Array(expandedData, originalData.byteLength * i).set(new Uint8Array(originalData));
  }

  // Fixup stride0TestData to use arrayStride 0.
  stride0TestData.attributes[0].offset = offset;
  stride0TestData.attributes[0].expectedData = expandedData;
  stride0TestData.attributes[0].testComponentCount *= kCount;
  stride0TestData.arrayStride = 0;
  stride0VertexBuffer.vbOffset = 0;

  // Create the part of the state that will be varying for each vertex / instance
  const varyingTestData = t.createTestAndPipelineData(
    [
    {
      slot: 1,
      arrayStride: 32,
      stepMode,
      attributes: [{ format, offset: 0, shaderLocation: 1 }]
    }],

    kCount,
    kCount
  )[0];
  const varyingVertexBuffer = t.createVertexBuffers([varyingTestData], kCount, kCount)[0];

  // Run the test with the merged test state.
  const state = [stride0TestData, varyingTestData];
  const vertexBuffers = [stride0VertexBuffer, varyingVertexBuffer];

  const pipeline = t.makeTestPipeline(state, kCount, kCount);
  const expectedDataBG = t.createExpectedBG(state, pipeline);
  t.submitRenderPass(pipeline, vertexBuffers, expectedDataBG, kCount, kCount);
});

g.test('discontiguous_location_and_attribs').
desc('Test that using far away slots / shaderLocations works as expected').
fn((t) => {
  t.runTest([
  {
    slot: t.device.limits.maxVertexBuffers - 1,
    arrayStride: 4,
    stepMode: 'vertex',
    attributes: [
    { format: 'uint8x2', offset: 2, shaderLocation: 0 },
    { format: 'uint8x2', offset: 0, shaderLocation: 8 }]

  },
  {
    slot: 1,
    arrayStride: 16,
    stepMode: 'instance',
    vbOffset: 1000,
    attributes: [
    {
      format: 'uint32x4',
      offset: 0,
      shaderLocation: t.device.limits.maxVertexAttributes - 1
    }]

  }]
  );
});

g.test('overlapping_attributes').
desc(
  `Test that overlapping attributes in the same vertex buffer works
   - Test for all formats`
).
params((u) => u.combine('format', kVertexFormats)).
fn((t) => {
  const { format } = t.params;

  // In compat mode, @builtin(vertex_index) and @builtin(instance_index) each take an attribute
  const maxVertexAttributes = t.device.limits.maxVertexAttributes - (t.isCompatibility ? 2 : 0);
  const attributes = [];
  for (let i = 0; i < maxVertexAttributes; i++) {
    attributes.push({ format, offset: 0, shaderLocation: i });
  }

  t.runTest([
  {
    slot: 0,
    stepMode: 'vertex',
    arrayStride: 32,
    attributes
  }]
  );
});