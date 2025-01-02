/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { anyOf } from '../../../../../util/compare.js';import { Type } from '../../../../../util/conversion.js';


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
  // We will populate a storage buffer with these input values laid out sequentially:
  // [ case0_input0, case0_input1, case0_input2, case0_input3, ...]
  //
  // The render pipeline will be launched several times over a viewport size of (2, 2). Each draw
  // call will execute a single quad (four fragment invocation), which will exercise one test case.
  // Each of these draw calls will use a different instance index, which is forwarded to the
  // fragment shader. Each invocation will determine its index into the storage buffer using its
  // fragment position and the instance index for that draw call.
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
  // calls the fwidth builtin with a value loaded from that fragment's index into the storage
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
  let inv_idx = u32(info.position.x) + u32(info.position.y)*2;
  let index = info.quad_idx*4 + inv_idx;
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
  const bufferSize = cases.length * 4 * valueStride;
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
  for (let i = 0; i < cases.length / vectorWidth; i++) {
    for (let v = 0; v < vectorWidth; v++) {
      const index = i * vectorWidth + v;
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
  for (let quad = 0; quad < cases.length / vectorWidth; quad++) {
    pass.draw(3, 1, undefined, quad);
  }
  pass.end();
  t.queue.submit([encoder.finish()]);

  // Check the outputs match the expected results.
  t.expectGPUBufferValuesPassCheck(
    outputBuffer,
    (outputData) => {
      for (let i = 0; i < cases.length / vectorWidth; i++) {
        for (let v = 0; v < vectorWidth; v++) {
          const index = i * vectorWidth + v;
          if (index >= cases.length) {
            break;
          }
          const c = cases[index];

          for (let x = 0; x < 4; x++) {
            if (non_uniform_discard && x === 0) {
              continue;
            }

            const index = (i * 4 + x) * valueStride + v * 4;
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