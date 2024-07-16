/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { Type } from '../../../../../util/conversion.js';
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
  // We will populate a storage buffer with these input values laid out sequentially:
  // [ case_0_input_1, case_0_input_0, case_1_input_1, case_1_input_0, ...]
  //
  // The render pipeline will be launched several times over a viewport size of (2, 2). Each draw
  // call will execute a single quad (four fragment invocation), which will exercise two test cases.
  // Each of these draw calls will use a different instance index, which is forwarded to the
  // fragment shader. Each invocation will determine its index into the storage buffer using its
  // fragment position and the instance index for that draw call.
  //
  // Consider two draw calls that test 4 cases (c_0, c_1, c_2, c_3).
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
  let valueStride = 4;
  let wgslType = 'f32';
  if (vectorize) {
    wgslType = `vec${vectorize}f`;
    valueStride = vectorize * 4;
    if (vectorize === 3) {
      valueStride = 16;
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

@group(0) @binding(0) var<storage, read> inputs : array<${wgslType}>;
@group(0) @binding(1) var<storage, read_write> outputs : array<${wgslType}>;

@fragment
fn frag(info : CaseInfo) {
  let case_idx = u32(info.position.${dir === 'x' ? 'y' : 'x'});
  let inv_idx = u32(info.position.${dir});
  let index = info.quad_idx*4 + case_idx*2 + inv_idx;
  let input = inputs[index];
  ${non_uniform_discard ? 'if inv_idx == 0 { discard; }' : ''}
  outputs[index] = ${builtin}(input);
}
`;

  // Create the render pipeline.
  const module = t.device.createShaderModule({ code });
  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: { module },
    fragment: { module, targets: [{ format: 'rgba8unorm', writeMask: 0 }] }
  });

  // Create storage buffers to hold the inputs and outputs.
  const bufferSize = cases.length * 2 * valueStride;
  const inputBuffer = t.createBufferTracked({
    size: bufferSize,
    usage: GPUBufferUsage.STORAGE,
    mappedAtCreation: true
  });
  const outputBuffer = t.createBufferTracked({
    size: bufferSize,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  });

  // Populate the input storage buffer with case input values.
  const valuesData = new Uint8Array(inputBuffer.getMappedRange());
  for (let i = 0; i < cases.length; i++) {
    const inputs = cases[i].input;
    inputs[0].copyTo(valuesData, (i * 2 + 1) * valueStride);
    inputs[1].copyTo(valuesData, i * 2 * valueStride);
  }
  inputBuffer.unmap();

  // Create a bind group for the storage buffers.
  const group = t.device.createBindGroup({
    entries: [
    { binding: 0, resource: { buffer: inputBuffer } },
    { binding: 1, resource: { buffer: outputBuffer } }],

    layout: pipeline.getBindGroupLayout(0)
  });

  // Create a texture to use as a color attachment.
  // We only need this for launching the desired number of fragment invocations.
  const colorAttachment = t.createTextureTracked({
    size: { width: 2, height: 2 },
    format: 'rgba8unorm',
    usage: GPUTextureUsage.RENDER_ATTACHMENT
  });

  // Submit the render pass to the device.
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: colorAttachment.createView(),
      loadOp: 'clear',
      storeOp: 'discard'
    }]

  });
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, group);
  for (let quad = 0; quad < cases.length / 2; quad++) {
    pass.draw(3, 1, undefined, quad);
  }
  pass.end();
  t.queue.submit([encoder.finish()]);

  // Check the outputs match the expected results.
  t.expectGPUBufferValuesPassCheck(
    outputBuffer,
    (outputData) => {
      for (let i = 0; i < cases.length; i++) {
        const c = cases[i];

        // Both invocations involved in the derivative should get the same result.
        for (let d = 0; d < 2; d++) {
          if (non_uniform_discard && d === 0) {
            continue;
          }

          const index = (i * 2 + d) * valueStride;
          const result = type.read(outputData, index);
          const cmp = toComparator(c.expected).compare(result);
          if (!cmp.matched) {
            // If this is a coarse derivative, the implementation is also allowed to calculate only
            // one of the two derivatives and return that result to all of the invocations.
            if (!builtin.endsWith('Fine')) {
              const c0 = cases[i % 2 === 0 ? i + 1 : i - 1];
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
      }
      return undefined;
    },
    {
      type: Uint8Array,
      typedLength: bufferSize
    }
  );
}