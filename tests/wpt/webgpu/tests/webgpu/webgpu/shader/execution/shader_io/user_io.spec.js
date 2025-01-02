/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test for user-defined shader I/O.

passthrough:
  * Data passed into vertex shader as uints and converted to test type
  * Passed from vertex to fragment as test type
  * Output from fragment shader as uint
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { range } from '../../../../common/util/util.js';
import { GPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

function generateInterstagePassthroughCode(type) {
  return `
${type === 'f16' ? 'enable f16;' : ''}
struct IOData {
  @builtin(position) pos : vec4f,
  @location(0) @interpolate(flat, either) user0 : ${type},
  @location(1) @interpolate(flat, either) user1 : vec2<${type}>,
  @location(2) @interpolate(flat, either) user2 : vec4<${type}>,
}

struct VertexInput {
  @builtin(vertex_index) idx : u32,
  @location(0) in0 : u32,
  @location(1) in1 : vec2u,
  @location(2) in2 : vec4u,
}

@vertex
fn vsMain(input : VertexInput) -> IOData {
  const vertices = array(
    vec4f(-1, -1, 0, 1),
    vec4f(-1,  1, 0, 1),
    vec4f( 1, -1, 0, 1),
  );
  var data : IOData;
  data.pos = vertices[input.idx];
  data.user0 = ${type}(input.in0);
  data.user1 = vec2<${type}>(input.in1);
  data.user2 = vec4<${type}>(input.in2);
  return data;
}

struct FragOutput {
  @location(0) out0 : u32,
  @location(1) out1 : vec2u,
  @location(2) out2 : vec4u,
}

@fragment
fn fsMain(input : IOData) -> FragOutput {
  var out : FragOutput;
  out.out0 = u32(input.user0);
  out.out1 = vec2u(input.user1);
  out.out2 = vec4u(input.user2);
  return out;
}
`;
}

function drawPassthrough(t, code) {
  // Default limit is 32 bytes of color attachments.
  // These attachments use 28 bytes (which is why vec3 is skipped).
  const formats = ['r32uint', 'rg32uint', 'rgba32uint'];
  const components = [1, 2, 4];
  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({ code }),
      entryPoint: 'vsMain',
      buffers: [
      {
        arrayStride: 4,
        attributes: [
        {
          format: 'uint32',
          offset: 0,
          shaderLocation: 0
        }]

      },
      {
        arrayStride: 8,
        attributes: [
        {
          format: 'uint32x2',
          offset: 0,
          shaderLocation: 1
        }]

      },
      {
        arrayStride: 16,
        attributes: [
        {
          format: 'uint32x4',
          offset: 0,
          shaderLocation: 2
        }]

      }]

    },
    fragment: {
      module: t.device.createShaderModule({ code }),
      entryPoint: 'fsMain',
      targets: formats.map((x) => {
        return { format: x };
      })
    },
    primitive: {
      topology: 'triangle-list'
    }
  });

  const vertexBuffer = t.makeBufferWithContents(
    new Uint32Array([
    // scalar: offset 0
    1, 1, 1, 0,
    // vec2: offset 16
    2, 2, 2, 2, 2, 2, 0, 0,
    // vec4: offset 48
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]
    ),
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.VERTEX
  );

  const bytesPerComponent = 4;
  // 256 is the minimum bytes per row for texture to buffer copies.
  const width = 256 / bytesPerComponent;
  const height = 2;
  const copyWidth = 4;
  const outputTextures = range(3, (i) =>
  t.createTextureTracked({
    size: [width, height],
    usage:
    GPUTextureUsage.COPY_SRC |
    GPUTextureUsage.RENDER_ATTACHMENT |
    GPUTextureUsage.TEXTURE_BINDING,
    format: formats[i]
  })
  );

  let bufferSize = 1;
  for (const comp of components) {
    bufferSize *= comp;
  }
  bufferSize *= outputTextures.length * bytesPerComponent * copyWidth;
  const outputBuffer = t.createBufferTracked({
    size: bufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: outputTextures.map((t) => ({
      view: t.createView(),
      loadOp: 'clear',
      storeOp: 'store'
    }))
  });
  pass.setPipeline(pipeline);
  pass.setVertexBuffer(0, vertexBuffer, 0, 12);
  pass.setVertexBuffer(1, vertexBuffer, 16, 24);
  pass.setVertexBuffer(2, vertexBuffer, 48, 48);
  pass.draw(3);
  pass.end();

  // Copy 'copyWidth' samples from each attachment into a buffer to check the results.
  let offset = 0;
  let expectArray = [];
  for (let i = 0; i < outputTextures.length; i++) {
    encoder.copyTextureToBuffer(
      { texture: outputTextures[i] },
      {
        buffer: outputBuffer,
        offset,
        bytesPerRow: bytesPerComponent * components[i] * width,
        rowsPerImage: height
      },
      { width: copyWidth, height: 1 }
    );
    offset += components[i] * bytesPerComponent * copyWidth;
    for (let j = 0; j < components[i]; j++) {
      const value = i + 1;
      expectArray = expectArray.concat([value, value, value, value]);
    }
  }
  t.queue.submit([encoder.finish()]);

  const expect = new Uint32Array(expectArray);
  t.expectGPUBufferValuesEqual(outputBuffer, expect);
}

g.test('passthrough').
desc('Tests passing user-defined data from vertex input through fragment output').
params((u) => u.combine('type', ['f32', 'f16', 'i32', 'u32'])).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const code = generateInterstagePassthroughCode(t.params.type);
  drawPassthrough(t, code);
});