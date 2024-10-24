/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert, iterRange } from '../../../../../../common/util/util.js';import { Float16Array } from '../../../../../../external/petamoriken/float16/float16.js';import { kTextureFormatInfo } from '../../../../../format_info.js';
import { GPUTest, TextureTestMixin } from '../../../../../gpu_test.js';

import { sparseScalarF16Range, sparseScalarF32Range, align } from '../../../../../util/math.js';
import { PRNG } from '../../../../../util/prng.js';

export class SubgroupTest extends TextureTestMixin(GPUTest) {}

export const kNumCases = 1000;
export const kStride = 128;

export const kWGSizes = [
[4, 1, 1],
[8, 1, 1],
[16, 1, 1],
[32, 1, 1],
[64, 1, 1],
[128, 1, 1],
[256, 1, 1],
[1, 4, 1],
[1, 8, 1],
[1, 16, 1],
[1, 32, 1],
[1, 64, 1],
[1, 128, 1],
[1, 256, 1],
[1, 1, 4],
[1, 1, 8],
[1, 1, 16],
[1, 1, 32],
[1, 1, 64],
[3, 3, 3],
[4, 4, 4],
[16, 16, 1],
[16, 1, 16],
[1, 16, 16],
[15, 3, 3],
[3, 15, 3],
[3, 3, 15]];


export const kPredicateCases = {
  every_even: {
    cond: `id % 2 == 0`,
    filter: (id, size) => {
      return id % 2 === 0;
    }
  },
  every_odd: {
    cond: `id % 2 == 1`,
    filter: (id, size) => {
      return id % 2 === 1;
    }
  },
  lower_half: {
    cond: `id < subgroupSize / 2`,
    filter: (id, size) => {
      return id < Math.floor(size / 2);
    }
  },
  upper_half: {
    cond: `id >= subgroupSize / 2`,
    filter: (id, size) => {
      return id >= Math.floor(size / 2);
    }
  },
  first_two: {
    cond: `id == 0 || id == 1`,
    filter: (id) => {
      return id === 0 || id === 1;
    }
  }
};

/**
 * Check the accuracy of the reduction operation.
 *
 * @param metadata An array containing subgroup ids for each invocation
 * @param output An array containing the results of the reduction for each invocation
 * @param indices An array of two values containing the indices of the interesting values in the input
 * @param values An array of two values containing the interesting values in the input
 * @param identity The identity for the operation
 * @param intervalGen A functor to generate an appropriate FPInterval for a binary operation
 */
function checkAccuracy(
metadata,
output,
indices,
values,
identity,
intervalGen)
{
  const subgroupIdIdx1 = metadata[indices[0]];
  const subgroupIdIdx2 = metadata[indices[1]];
  for (let i = 0; i < output.length; i++) {
    const subgroupId = metadata[i];

    const v1 = subgroupId === subgroupIdIdx1 ? values[0] : identity;
    const v2 = subgroupId === subgroupIdIdx2 ? values[1] : identity;
    const interval = intervalGen(v1, v2);
    if (!interval.contains(output[i])) {
      return new Error(`Invocation ${i}, subgroup id ${subgroupId}: incorrect result
- interval: ${interval.toString()}
- output: ${output[i]}`);
    }
  }

  return undefined;
}

/**
 * Run a floating-point accuracy subgroup test.
 *
 * @param t The base test
 * @param seed A seed for the PRNG
 * @param wgSize An array for the workgroup size
 * @param operation The subgroup operation
 * @param type The type (f16 or f32)
 * @param identity The identity for the operation
 * @param intervalGen A functor to generate an appropriate FPInterval for a binary operation
 */
export async function runAccuracyTest(
t,
seed,
wgSize,
operation,
type,
identity,
intervalGen)
{
  assert(seed < kNumCases);
  const prng = new PRNG(seed);

  // Compatibility mode has lower workgroup limits.
  const wgThreads = wgSize[0] * wgSize[1] * wgSize[2];
  const {
    maxComputeInvocationsPerWorkgroup,
    maxComputeWorkgroupSizeX,
    maxComputeWorkgroupSizeY,
    maxComputeWorkgroupSizeZ
  } = t.device.limits;
  t.skipIf(
    maxComputeInvocationsPerWorkgroup < wgThreads ||
    maxComputeWorkgroupSizeX < wgSize[0] ||
    maxComputeWorkgroupSizeY < wgSize[1] ||
    maxComputeWorkgroupSizeZ < wgSize[2],
    'Workgroup size too large'
  );

  // Bias half the cases to lower indices since most subgroup sizes are <= 64.
  let indexLimit = kStride;
  if (seed < kNumCases / 4) {
    indexLimit = 16;
  } else if (seed < kNumCases / 2) {
    indexLimit = 64;
  }

  // Ensure two distinct indices are picked.
  const idx1 = prng.uniformInt(indexLimit);
  let idx2 = prng.uniformInt(indexLimit - 1);
  if (idx1 === idx2) {
    idx2++;
  }
  assert(idx2 < indexLimit);

  // Select two random values.
  const range = type === 'f16' ? sparseScalarF16Range() : sparseScalarF32Range();
  const numVals = range.length;
  const val1 = range[prng.uniformInt(numVals)];
  const val2 = range[prng.uniformInt(numVals)];

  const extraEnables = type === 'f16' ? `enable f16;\nenable subgroups_f16;` : ``;
  const wgsl = `
enable subgroups;
${extraEnables}

@group(0) @binding(0)
var<storage> inputs : array<${type}>;

@group(0) @binding(1)
var<storage, read_write> outputs : array<${type}>;

struct Metadata {
  subgroup_id : array<u32, ${kStride}>,
}

@group(0) @binding(2)
var<storage, read_write> metadata : Metadata;

@compute @workgroup_size(${wgSize[0]}, ${wgSize[1]}, ${wgSize[2]})
fn main(
  @builtin(local_invocation_index) lid : u32,
) {
  metadata.subgroup_id[lid] = subgroupBroadcast(lid, 0);
  outputs[lid] = ${operation}(inputs[lid]);
}`;

  const inputData =
  type === 'f16' ?
  new Float16Array([
  ...iterRange(kStride, (x) => {
    if (x === idx1) return val1;
    if (x === idx2) return val2;
    return identity;
  })]
  ) :
  new Float32Array([
  ...iterRange(kStride, (x) => {
    if (x === idx1) return val1;
    if (x === idx2) return val2;
    return identity;
  })]
  );

  const inputBuffer = t.makeBufferWithContents(
    inputData,
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
  );
  t.trackForCleanup(inputBuffer);

  const outputBuffer = t.makeBufferWithContents(
    new Float32Array([...iterRange(kStride, (x) => 0)]),
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
  );
  t.trackForCleanup(outputBuffer);

  const numMetadata = kStride;
  const metadataBuffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(numMetadata, (x) => 0)]),
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
  );

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: wgsl
      }),
      entryPoint: 'main'
    }
  });
  const bg = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: {
        buffer: inputBuffer
      }
    },
    {
      binding: 1,
      resource: {
        buffer: outputBuffer
      }
    },
    {
      binding: 2,
      resource: {
        buffer: metadataBuffer
      }
    }]

  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.dispatchWorkgroups(1, 1, 1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  const metadataReadback = await t.readGPUBufferRangeTyped(metadataBuffer, {
    srcByteOffset: 0,
    type: Uint32Array,
    typedLength: numMetadata,
    method: 'copy'
  });
  const metadata = metadataReadback.data;

  let output;
  if (type === 'f16') {
    const outputReadback = await t.readGPUBufferRangeTyped(outputBuffer, {
      srcByteOffset: 0,
      type: Float16Array,
      typedLength: kStride,
      method: 'copy'
    });
    output = outputReadback.data;
  } else {
    const outputReadback = await t.readGPUBufferRangeTyped(outputBuffer, {
      srcByteOffset: 0,
      type: Float32Array,
      typedLength: kStride,
      method: 'copy'
    });
    output = outputReadback.data;
  }

  t.expectOK(checkAccuracy(metadata, output, [idx1, idx2], [val1, val2], identity, intervalGen));
}

// Repeat the bit pattern evey 16 bits for use with 16-bit types.
export const kDataSentinel = 999 | 999 << 16;

/**
 * Runs compute shader subgroup test
 *
 * The test makes the following assumptions:
 * * group(0) binding(0) is a storage buffer for input data
 * * group(0) binding(1) is an output storage buffer for outputUintsPerElement * wgSize uints
 * * group(0) binding(2) is an output storage buffer for 2 * wgSize uints
 *
 * @param t The base test
 * @param wgsl The shader code
 * @param outputUintsPerElement number of uints output per invocation
 * @param inputData the input data
 * @param checkFunction a functor that takes the output storage buffer data to check result validity
 */
export async function runComputeTest(
t,
wgsl,
wgSize,
outputUintsPerElement,
inputData,
checkFunction)
{
  // Compatibility mode has lower workgroup limits.
  const wgThreads = wgSize[0] * wgSize[1] * wgSize[2];
  const {
    maxComputeInvocationsPerWorkgroup,
    maxComputeWorkgroupSizeX,
    maxComputeWorkgroupSizeY,
    maxComputeWorkgroupSizeZ
  } = t.device.limits;
  t.skipIf(
    maxComputeInvocationsPerWorkgroup < wgThreads ||
    maxComputeWorkgroupSizeX < wgSize[0] ||
    maxComputeWorkgroupSizeY < wgSize[1] ||
    maxComputeWorkgroupSizeZ < wgSize[2],
    'Workgroup size too large'
  );

  const inputBuffer = t.makeBufferWithContents(
    inputData,
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
  );
  t.trackForCleanup(inputBuffer);

  const outputUints = outputUintsPerElement * wgThreads;
  const outputBuffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(outputUints, (x) => kDataSentinel)]),
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
  );
  t.trackForCleanup(outputBuffer);

  const numMetadata = 2 * wgThreads;
  const metadataBuffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(numMetadata, (x) => kDataSentinel)]),
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
  );

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: wgsl
      })
    }
  });
  const bg = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: {
        buffer: inputBuffer
      }
    },
    {
      binding: 1,
      resource: {
        buffer: outputBuffer
      }
    },
    {
      binding: 2,
      resource: {
        buffer: metadataBuffer
      }
    }]

  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.dispatchWorkgroups(1, 1, 1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  const metadataReadback = await t.readGPUBufferRangeTyped(metadataBuffer, {
    srcByteOffset: 0,
    type: Uint32Array,
    typedLength: numMetadata,
    method: 'copy'
  });
  const metadata = metadataReadback.data;

  const outputReadback = await t.readGPUBufferRangeTyped(outputBuffer, {
    srcByteOffset: 0,
    type: Uint32Array,
    typedLength: outputUints,
    method: 'copy'
  });
  const output = outputReadback.data;

  t.expectOK(checkFunction(metadata, output));
}

// Minimum size is [3, 3].
export const kFramebufferSizes = [
[15, 15],
[16, 16],
[17, 17],
[19, 13],
[13, 10],
[111, 3],
[3, 111],
[35, 3],
[3, 35],
[53, 13],
[13, 53],
[3, 3]];


/**
 * Runs a subgroup builtin test for fragment shaders
 *
 * This test draws a full screen triangle.
 * Tests should avoid checking the last row or column to avoid helper
 * invocations. Underlying APIs do not consistently guarantee whether
 * helper invocations participate in subgroup operations.
 * @param t The base test
 * @param format The framebuffer format
 * @param fsShader The fragment shader with the following interface:
 *                 Location 0 output is framebuffer with format
 *                 Group 0 binding 0 is input data
 * @param width The framebuffer width
 * @param height The framebuffer height
 * @param inputData The input data
 * @param checker A functor to check the framebuffer values
 */
export async function runFragmentTest(
t,
format,
fsShader,
width,
height,
inputData,
checker)
{
  const vsShader = `
@vertex
fn vsMain(@builtin(vertex_index) index : u32) -> @builtin(position) vec4f {
  const vertices = array(
    vec2(-2, 4), vec2(-2, -4), vec2(2, 0),
  );
  return vec4f(vec2f(vertices[index]), 0, 1);
}`;

  assert(width >= 3, 'Minimum width is 3');
  assert(height >= 3, 'Minimum height is 3');
  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({ code: vsShader })
    },
    fragment: {
      module: t.device.createShaderModule({ code: fsShader }),
      targets: [{ format }]
    },
    primitive: {
      topology: 'triangle-list'
    }
  });

  const { blockWidth, blockHeight, bytesPerBlock } = kTextureFormatInfo[format];
  assert(bytesPerBlock !== undefined);

  const blocksPerRow = width / blockWidth;
  const blocksPerColumn = height / blockHeight;
  // 256 minimum arises from image copy requirements.
  const bytesPerRow = align(blocksPerRow * (bytesPerBlock ?? 1), 256);
  const byteLength = bytesPerRow * blocksPerColumn;
  const uintLength = byteLength / 4;

  const buffer = t.makeBufferWithContents(
    inputData,
    GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST
  );

  const bg = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: {
        buffer
      }
    }]

  });

  const framebuffer = t.createTextureTracked({
    size: [width, height],
    usage:
    GPUTextureUsage.COPY_SRC |
    GPUTextureUsage.COPY_DST |
    GPUTextureUsage.RENDER_ATTACHMENT |
    GPUTextureUsage.TEXTURE_BINDING,
    format
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: framebuffer.createView(),
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.draw(3);
  pass.end();
  t.queue.submit([encoder.finish()]);

  const copyBuffer = t.copyWholeTextureToNewBufferSimple(framebuffer, 0);
  const readback = await t.readGPUBufferRangeTyped(copyBuffer, {
    srcByteOffset: 0,
    type: Uint32Array,
    typedLength: uintLength,
    method: 'copy'
  });
  const data = readback.data;

  t.expectOK(checker(data));
}