/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { Type } from '../../../../../util/conversion.js';import { align } from '../../../../../util/math.js';

import { toComparator } from '../../expectation.js';
import { packScalarsToVector } from '../../expression.js';

/**
 * Run a test for a derivative builtin function.
 * @param t the GPUTest
 * @param cases list of test cases to run
 * @param builtin the builtin function to test
 * @param non_uniform_discard if true, one of each pair of invocations will discard
 * @param vectorize if defined, the vector width to use (2, 3, or 4)
 */
export function runDerivativeTest(
t,
cases,
builtin,
non_uniform_discard,
vectorize)
{
  // If the 'vectorize' config option was provided, pack the cases into vectors.
  let type = Type.f32;
  if (vectorize !== undefined) {
    const packed = packScalarsToVector([type, type], type, cases, vectorize);
    cases = packed.cases;
    type = packed.resultType;
  }

  ////////////////////////////////////////////////////////////////
  // The two input values for a given case are distributed to two different invocations in a quad.
  // We will populate a uniform buffer with these input values laid out sequentially:
  // [ case_0_input_1, case_0_input_0, case_1_input_1, case_1_input_0, ...]
  //
  // The render pipeline will be launched once per pair of cases over a viewport
  // size of (2, 2). Each 2x2 set of calls will will exercise two test cases.
  // Each of these draw calls will use a different instance index, which is
  // forwarded to the fragment shader. Each invocation returns the result which
  // is stored in a rgba32uint texture.
  //
  // Consider draw calls that test 4 cases (c_0, c_1, c_2, c_3).
  //
  // For derivatives along the 'x' direction, the mapping from fragment position to case input is:
  // Quad 0: | c_0_i_1 | c_0_i_0 |     Quad 1: | c_2_i_1 | c_2_i_0 |
  //         | c_1_i_1 | c_1_i_0 |             | c_3_i_1 | c_3_i_0 |
  //
  // For derivatives along the 'y' direction, the mapping from fragment position to case input is:
  // Quad 0: | c_0_i_1 | c_1_i_1 |     Quad 1: | c_2_i_1 | c_3_i_1 |
  //         | c_0_i_0 | c_1_i_0 |             | c_2_i_0 | c_3_i_0 |
  //
  ////////////////////////////////////////////////////////////////

  // Determine the direction of the derivative ('x' or 'y') from the builtin name.
  const dir = builtin[3];

  // Determine the WGSL type to use in the shader, and the stride in bytes between values.
  const valueStride = 16;
  let conversionFromInput = 'input.x';
  let conversionToOutput = `vec4f(v)`;
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

  // Define a vertex shader that draws a triangle over the full viewport, and a fragment shader that
  // calls the derivative builtin with a value loaded from that fragment's index into the storage
  // buffer (determined using the quad index and fragment position, as described above).
  const code = `
struct CaseInfo {
  @builtin(position) position: vec4f,
  @location(0) @interpolate(flat, either) quad_idx: u32,
}

@vertex
fn vert(@builtin(vertex_index) vertex_idx: u32,
        @builtin(instance_index) instance_idx: u32) -> CaseInfo {
  const kVertices = array(
    vec2f(-2, -2),
    vec2f( 2, -2),
    vec2f( 0,  2),
  );
  return CaseInfo(vec4(kVertices[vertex_idx], 0, 1), instance_idx);
}

@group(0) @binding(0) var<uniform> inputs : array<vec4f, ${cases.length * 2}>;

@fragment
fn frag(info : CaseInfo) -> @location(0) vec4u {
  let case_idx = u32(info.position.${dir === 'x' ? 'y' : 'x'});
  let inv_idx = u32(info.position.${dir});
  let index = info.quad_idx*4 + case_idx*2 + inv_idx;
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

  // Create storage buffers to hold the inputs and outputs.
  const bufferSize = cases.length * 2 * valueStride;
  const inputBuffer = t.createBufferTracked({
    size: bufferSize,
    usage: GPUBufferUsage.UNIFORM,
    mappedAtCreation: true
  });

  // Populate the input uniform buffer with case input values.
  const valuesData = new Uint8Array(inputBuffer.getMappedRange());
  for (let i = 0; i < cases.length; i++) {
    const inputs = cases[i].input;
    inputs[0].copyTo(valuesData, (i * 2 + 1) * valueStride);
    inputs[1].copyTo(valuesData, i * 2 * valueStride);
  }
  inputBuffer.unmap();

  // Create a bind group for the storage buffers.
  const group = t.device.createBindGroup({
    entries: [{ binding: 0, resource: { buffer: inputBuffer } }],
    layout: pipeline.getBindGroupLayout(0)
  });

  const colorAttachment = t.createTextureTracked({
    size: { width: 2, height: 2 },
    format: 'rgba32uint',
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
  });
  const bytesPerRow = align(valueStride * colorAttachment.width, 256);

  // Submit the render pass to the device.
  const results = [];
  const encoder = t.device.createCommandEncoder({ label: 'runDerivativeTest' });
  for (let quad = 0; quad < cases.length / 2; quad++) {
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
    pass.draw(3, 1, 0, quad);
    pass.end();
    const outputBuffer = t.createBufferTracked({
      size: bytesPerRow * colorAttachment.height,
      usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.COPY_SRC
    });
    results.push(outputBuffer);
    encoder.copyTextureToBuffer(
      { texture: colorAttachment },
      { buffer: outputBuffer, bytesPerRow },
      [colorAttachment.width, colorAttachment.height]
    );
  }

  t.queue.submit([encoder.finish()]);

  // Check the outputs match the expected results.
  results.forEach((outputBuffer, quadNdx) => {
    t.expectGPUBufferValuesPassCheck(
      outputBuffer,
      (outputData) => {
        for (let i = 0; i < 4; ++i) {
          const tx = i % 2;
          const ty = i / 2 | 0;
          const [inputNdx, caseNdx] = dir === 'x' ? [tx, ty] : [ty, tx];
          const caseNdxAlt = 1 - caseNdx;
          const c = cases[quadNdx * 2 + caseNdx];

          // Both invocations involved in the derivative should get the same result.
          if (non_uniform_discard && inputNdx === 0) {
            continue;
          }

          const index = ty * bytesPerRow + tx * valueStride;
          const result = type.read(outputData, index);
          const cmp = toComparator(c.expected).compare(result);
          if (!cmp.matched) {
            // If this is a coarse derivative, the implementation is also allowed to calculate only
            // one of the two derivatives and return that result to all of the invocations.
            if (!builtin.endsWith('Fine')) {
              const c0 = cases[quadNdx * 2 + caseNdxAlt];
              const cmp0 = toComparator(c0.expected).compare(result);
              if (!cmp0.matched) {
                return new Error(`
  1st pair: (${c.input.join(', ')})
  expected: ${cmp.expected}

  2nd pair: (${c0.input.join(', ')})
  expected: ${cmp0.expected}

  returned: ${result}`);
              }
            } else {
              return new Error(`
    inputs: (${c.input.join(', ')})
  expected: ${cmp.expected}

  returned: ${result}`);
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