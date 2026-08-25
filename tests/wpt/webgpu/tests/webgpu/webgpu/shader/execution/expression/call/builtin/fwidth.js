/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert } from '../../../../../../common/util/util.js';import { anyOf } from '../../../../../util/compare.js';
import { Type } from '../../../../../util/conversion.js';

import { align } from '../../../../../util/math.js';

import { toComparator } from '../../expectation.js';

/**
 * Run a test for a fwidth builtin function.
 * @param t the GPUTest
 * @param cases list of test cases to run
 * @param builtin the builtin function to test
 * @param non_uniform_discard if true, one of each pair of invocations will discard
 * @param vectorize if defined, the vector width to use (2, 3, or 4)
 */
export function runFWidthTest(
t,
cases,
builtin,
non_uniform_discard,
vectorize)
{
  ////////////////////////////////////////////////////////////////
  // The four input values for a given case are distributed to across the invocations in a quad.
  // We will populate a uniform buffer with these input values laid out sequentially:
  // [ case0_input0, case0_input1, case0_input2, case0_input3, ...]
  //
  // The render pipeline a 512x2 texture. In the fragment shader, every 2x2 texels is one test case.
  // The results are the output from the fragment shader.
  //
  // Consider two draw calls that test 2 cases (c0, c1).
  //
  // The mapping from fragment position to case input is:
  // Quad 0: | c0_i0 | c0_i1 |     Quad 1: | c1_i0 | c1_i1 |
  //         | c0_i2 | c0_i3 |             | c1_i2 | c1_i3 |
  //
  ////////////////////////////////////////////////////////////////

  // If the 'vectorize' config option was provided, pack the cases into vectors.
  let vectorWidth = 1;
  if (vectorize !== undefined) {
    vectorWidth = vectorize;
  }

  // Determine the WGSL type to use in the shader, and the stride in bytes between values.
  const valueStride = 16;
  let conversionFromInput = 'input.x';
  let conversionToOutput = `vec4f(v, 0, 0, 0)`;
  if (vectorize) {
    switch (vectorize) {
      case 2:
        conversionFromInput = 'input.xy';
        conversionToOutput = 'vec4f(v, 0, 0)';
        break;
      case 3:
        conversionFromInput = 'input.xyz';
        conversionToOutput = 'vec4f(v, 0)';
        break;
      case 4:
        conversionFromInput = 'input';
        conversionToOutput = 'v';
        break;
    }
  }

  const kUniformBufferSize = 16384; // min supported by compat mode.
  const kNumCasesPerUniformBuffer = kUniformBufferSize / 64;

  // Define a vertex shader that draws a triangle over the full viewport, and a fragment shader that
  // calls the fwidth builtin with a value loaded from that fragment's index into the storage
  // buffer (determined using the quad index and fragment position, as described above).
  const code = `
@vertex
fn vert(@builtin(vertex_index) vertex_idx: u32) -> @builtin(position) vec4f {
  const kVertices = array(
    vec2f( 3, -1),
    vec2f(-1,  3),
    vec2f(-1, -1),
  );
  return vec4(kVertices[vertex_idx], 0, 1);
}

@group(0) @binding(0) var<uniform> inputs : array<vec4f, ${kNumCasesPerUniformBuffer * 4}>;

@fragment
fn frag(@builtin(position) position: vec4f) -> @location(0) vec4u {
  let t = vec2u(position.xy);
  let inv_idx = t.x % 2 + (t.y % 2) * 2;
  let q = t / 2;
  let quad_idx = q.y * 256 + q.x;
  let index = quad_idx * 4 + inv_idx;
  let input = inputs[index];
  ${non_uniform_discard ? 'if inv_idx == 0 { discard; }' : ''}
  let v = ${builtin}(${conversionFromInput});
  return bitcast<vec4u>(${conversionToOutput});
}
`;

  // Create the render pipeline.
  const module = t.device.createShaderModule({ code });
  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: { module },
    fragment: { module, targets: [{ format: 'rgba32uint' }] }
  });

  // Create a texture to use as a color attachment to receive the results;
  const width = kNumCasesPerUniformBuffer * 2;
  const height = 2;
  // note: We could limit it to this size and increase height but kNumCasesPerUniformBuffer is limited to 256
  // because we can't fit more into a single uniform buffer in compat.
  assert(width < t.device.limits.maxTextureDimension2D);
  const colorAttachment = t.createTextureTracked({
    size: [width, height],
    format: 'rgba32uint',
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
  });
  const bytesPerRow = align(width * 16, 256);

  const results = [];
  const encoder = t.device.createCommandEncoder({ label: 'runFWidthTest' });
  for (let c = 0; c < cases.length; c += kNumCasesPerUniformBuffer) {
    // Create uniform buffer to hold the inputs.
    const inputBuffer = t.createBufferTracked({
      size: kUniformBufferSize,
      usage: GPUBufferUsage.UNIFORM,
      mappedAtCreation: true
    });
    const valuesData = new Uint8Array(inputBuffer.getMappedRange());

    // Populate the input uniform buffer with case input values.
    for (let i = 0; i < kNumCasesPerUniformBuffer / vectorWidth; i++) {
      for (let v = 0; v < vectorWidth; v++) {
        const index = c + i * vectorWidth + v;
        if (index >= cases.length) {
          break;
        }
        const inputs = cases[index].input;
        for (let x = 0; x < 4; x++) {
          inputs[x].copyTo(valuesData, (i * 4 + x) * valueStride + v * 4);
        }
      }
    }
    inputBuffer.unmap();

    // Create a bind group for the input buffer.
    const group = t.device.createBindGroup({
      entries: [{ binding: 0, resource: { buffer: inputBuffer } }],
      layout: pipeline.getBindGroupLayout(0)
    });

    // Submit the render pass to the device.
    const pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: colorAttachment.createView(),
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, group);
    pass.draw(3);
    pass.end();

    // Create buffer to hold the outputs.
    const outputBuffer = t.createBufferTracked({
      size: bytesPerRow * height,
      usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.COPY_SRC
    });
    results.push(outputBuffer);

    // Copy the texture to the output buffer
    encoder.copyTextureToBuffer(
      { texture: colorAttachment },
      { buffer: outputBuffer, bytesPerRow },
      [colorAttachment.width, colorAttachment.height]
    );
  }
  t.queue.submit([encoder.finish()]);

  results.forEach((outputBuffer, groupNdx) => {
    // Check the outputs match the expected results.
    t.expectGPUBufferValuesPassCheck(
      outputBuffer,
      (outputData) => {
        const base = groupNdx * kNumCasesPerUniformBuffer;
        const numCases = Math.min(kNumCasesPerUniformBuffer, cases.length - base);
        const numQuads = numCases / vectorWidth;
        for (let i = 0; i < numQuads; i++) {
          for (let v = 0; v < vectorWidth; v++) {
            const caseNdx = base + i * vectorWidth + v;
            if (caseNdx >= cases.length) {
              break;
            }
            const c = cases[caseNdx];

            for (let x = 0; x < 4; x++) {
              if (non_uniform_discard && x === 0) {
                continue;
              }

              const tx = x % 2;
              const ty = x / 2 | 0;
              const index = ty * bytesPerRow + i * 32 + tx * 16 + v * 4;
              const result = Type.f32.read(outputData, index);

              let expected = c.expected;
              if (builtin.endsWith('Fine')) {
                expected = toComparator(expected[x]);
              } else {
                expected = anyOf(...expected);
              }

              const cmp = expected.compare(result);
              if (!cmp.matched) {
                return new Error(`
     caseNdx: ${caseNdx} v: ${v} x: ${x}
      inputs: (${c.input.join(', ')})
    expected: ${cmp.expected}

    returned: ${result}`);
              }
            }
          }
        }
        return undefined;
      },
      {
        type: Uint8Array,
        typedLength: outputBuffer.size
      }
    );
  });
}