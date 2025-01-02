/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests writes from different invocations to adjacent scalars do not interfere.
This is especially interesting when the scalar type is narrower than 32-bits.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';
import { PRNG } from '../../../util/prng.js';

export const g = makeTestGroup(GPUTest);

// Algorithm: with N invocations, N is even:
//     srcBuffer: An array of random scalar values.  Avoids unsupported values like infinity and NaN.
//     resultBuffer: A result array
//     pattern: 0|1|2|3
//       Pattern 0: Identity: invocation i: dst[i] = src[i]
//       Pattern 1: Try to prevent write coalescing.
//          Even elements stay in place.
//          Reverse order of odd elements.
//          invocation 2k:   dst[2k] = src[2k]
//          invocation 2k+1: dst[2k+1] = src[N - (2k+1)]
//          Example: with N=6
//             dst[0] = src[0]
//             dst[1] = src[5]
//             dst[2] = src[2]
//             dst[3] = src[3]
//             dst[4] = src[4]
//             dst[5] = src[1]
//       Pattern 2: Try to prevent write coalescing.
//          Reverse order of even elements.
//          Odd elements stay in place.
//          invocation 2k:   dst[2k] = src[N - 2 - 2k]
//          invocation 2k+1: dst[2k+1] = src[2k+1]
//          Example: with N=6
//             dst[0] = src[4]
//             dst[1] = src[1]
//             dst[2] = src[2]
//             dst[3] = src[3]
//             dst[4] = src[0]
//             dst[5] = src[5]
//       Pattern 3: Reverse elements: dst[i] = src[N-1-i]
//     addressSpace: workgroup|storage
//          Where dst is allocated.



const kAddressSpaces = ['workgroup', 'storage'];
const kPatterns = [0, 1, 2, 3];








// For simplicity, make the entire source (and destination) array fit
// in workgroup memory.
// We can count on up to 16384 bytes in workgroup memory.
const kNumValues = 4096; // Assumed even
const kWorkgroupSize = 128; // Use 1-dimensional workgroups.

/**
 * @returns an integer for the bit pattern of a random finite f16 value.
 * Consumes values from `prng`.
 *
 * @param prng - a pseudo-random number generator.
 */
function randomFiniteF16(prng) {
  const exponent_bits = 0x7c00;
  // With any reasonable random number stream, the average number
  // of trips around this loop is < 1 + 1/32 because there are 5
  // exponent bits.
  let candidate;
  do {
    candidate = prng.randomU32() & 0xffff;
    // Non-finite f16 values have all 1 bits in the exponent.
  } while ((candidate & exponent_bits) === exponent_bits);
  return candidate;
}

/**
 * Fills array `arr` with random finite f16 values.
 * Consumes values from `prng`.
 *
 * @param prng - a pseudo-random number generator.
 * @param arr - the array to fill. Assume it is already correctly sized.
 */
function fillWithRandomFiniteF16(prng, arr) {
  for (let i = 0; i < arr.length; i++) {
    arr[i] = randomFiniteF16(prng);
  }
}

/**
 * @returns the expression for the destination index, based on `pattern`.
 *
 * @param i the WGSL string for the source index
 * @param pattern the indexing pattern
 */
function getDstIndexExpression(i, pattern) {
  switch (pattern) {
    case 0:
      return `${i}`;
    case 1:
      // Even elements map to themselves.
      // Odd elements map to the reversed order of odd elements.
      return `select(${kNumValues} - ${i}, ${i}, (${i} & 1) == 0)`;
    case 2:
      // Even elements map to the reversed order of odd elements.
      // Since N is even, element 0 should get index N-2. (!)
      // Odd elements map to themselves.
      return `select(${i}, ${kNumValues} - 2 - ${i}, (${i} & 1) == 0)`;
    case 3:
      return `${kNumValues} - 1 -${i}`;
  }
}

/**
 * Computes the reference (correct) result for the given source array and indexing pattern.
 *
 * @param pattern the indexing pattern
 * @param src the source array
 * @param dst the array to fill with values transferred from `src`
 */
function computeReference(pattern, src, dst) {
  for (let i = 0; i < src.length; i++) {
    const isEven = (i & 1) === 0;
    switch (pattern) {
      case 0:
        dst[i] = src[i];
        break;
      case 1:
        if (isEven) {
          dst[i] = src[i];
        } else {
          dst[src.length - i] = src[i];
        }
        break;
      case 2:
        if (isEven) {
          dst[kNumValues - 2 - i] = src[i];
        } else {
          dst[i] = src[i];
        }
        break;
      case 3:
        dst[src.length - 1 - i] = src[i];
        break;
    }
  }
}

/**
 * @returns the source text for a shader that copies elements from a source
 * buffer to a destination buffer, while remapping indices according to the
 * specified pattern.
 *
 * @param p contains the address space and pattern
 */
function makeShaderText(p) {
  // When the destination buffer is in 'storage', then write directly to it.
  // Otherwise, destination is in workgroup memory, and we need to name the
  // output buffer differently.
  const dstBuf = p.addressSpace === 'storage' ? 'dst' : 'dstBuf';

  const parts = [];

  parts.push(`
    enable f16;
    @group(0) @binding(0) var<storage> src: array<f16>;
    @group(0) @binding(1) var<storage,read_write> ${dstBuf}: array<f16>;
    `);

  if (p.addressSpace === 'workgroup') {
    parts.push(`var<workgroup> dst: array<f16,${kNumValues}>;`);
  }

  parts.push(`
    @compute @workgroup_size(${kWorkgroupSize})
    fn adjacent_writes(@builtin(global_invocation_id) gid: vec3u) {
        let srcIndex = gid.x;
        let dstIndex = ${getDstIndexExpression('srcIndex', p.pattern)};
        dst[dstIndex] = src[srcIndex];
    `);

  if (p.addressSpace === 'workgroup') {
    // Copy to the output buffer.
    // The barrier is not necessary here, but it should prevent
    // the compiler from being clever and optimizing away the
    // intermediate write to workgroup memory.
    parts.push(`        workgroupBarrier();`);
    parts.push(`        ${dstBuf}[dstIndex] = dst[dstIndex];`);
  }
  parts.push('}');

  return parts.join('\n');
}

/**
 * Runs the test on the GPU, generating random source data and
 * checking the results against the expected permutation of that data.
 *
 * @param t the AdjacentWritesTest specification.
 */
function runTest(t) {
  const seed = (t.params.pattern + 1) * t.params.addressSpace.length;
  const prng = new PRNG(seed);

  const expected = new Uint16Array(kNumValues);

  const bytesPerScalar = 2; // f16 is 2 bytes wide.
  const bufByteSize = kNumValues * bytesPerScalar;
  const hostSrcBuf = t.createBufferTracked({
    size: bufByteSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.MAP_WRITE,
    mappedAtCreation: true
  });
  {
    const hostSrcUint16 = new Uint16Array(hostSrcBuf.getMappedRange());
    fillWithRandomFiniteF16(prng, hostSrcUint16);
    computeReference(t.params.pattern, hostSrcUint16, expected);
    hostSrcBuf.unmap();
  }

  const srcBuf = t.createBufferTracked({
    size: bufByteSize,
    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
  });
  const dstBuf = t.createBufferTracked({
    size: bufByteSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  });

  const shaderText = makeShaderText(t.params);
  const shader = t.device.createShaderModule({ code: shaderText });
  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: { module: shader, entryPoint: 'adjacent_writes' }
  });
  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    { binding: 0, resource: { buffer: srcBuf } },
    { binding: 1, resource: { buffer: dstBuf } }]

  });

  const encoder = t.device.createCommandEncoder();
  encoder.copyBufferToBuffer(hostSrcBuf, 0, srcBuf, 0, bufByteSize);

  const computeEncoder = encoder.beginComputePass();
  computeEncoder.setPipeline(pipeline);
  computeEncoder.setBindGroup(0, bindGroup);
  computeEncoder.dispatchWorkgroups(kNumValues / kWorkgroupSize);
  computeEncoder.end();

  const commands = encoder.finish();
  t.device.queue.submit([commands]);

  t.expectGPUBufferValuesEqual(dstBuf, expected);
}

g.test('f16').
desc(
  `Check that writes by different invocations to adjacent f16 values in an array do not interfere with each other.`
).
params((u) => u.combine('addressSpace', kAddressSpaces).combine('pattern', kPatterns)).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('shader-f16');
}).
fn((t) => runTest(t));