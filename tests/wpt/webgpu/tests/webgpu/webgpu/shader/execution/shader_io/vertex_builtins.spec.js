/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Test vertex shader builtin variables

* test builtin(clip_distances)
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest, TextureTestMixin } from '../../../gpu_test.js';

class VertexBuiltinTest extends TextureTestMixin(GPUTest) {}

export const g = makeTestGroup(VertexBuiltinTest);

g.test('outputs,clip_distances').
desc(
  `
    Test vertex shader builtin(clip_distances) values.

    In the tests, we draw a square with two triangles (top-right and bottom left), whose vertices
    have different clip distances values. (Top Left: -1, Bottom Right: 1 Top Right & Bottom Left: 0)
    1. The clip distances values of the pixels in the top-left region should be less than 0 so these
       pixels will all be invisible
    2. The clip distances values of the pixels on the top-right-to-bottom-left diagonal line should
       be equal to 0
    3. The clip distances values of the pixels in the bottom-right region should be greater than 0

    -1 - - - - - 0
     | \\      x x
     |   \\  x x x
     |    \\ x x x
     |   x x\\ x x
     | x x x x\\ x
     0 x x x x x 1
  `
).
params((u) => u.combine('clipDistances', [1, 2, 3, 4, 5, 6, 7, 8])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('clip-distances');
}).
fn((t) => {
  const { clipDistances } = t.params;

  // Draw two triangles (top-right and bottom left) into Red, whose vertices have different clip
  // distances values. (Top Left: -1, Bottom Right: 1 Top Right & Bottom Left: 0)
  const code = `
    enable clip_distances;
    const kClipDistancesSize = ${clipDistances};
    struct VertexOutputs {
        @builtin(position) position : vec4f,
        @builtin(clip_distances) clipDistances : array<f32, kClipDistancesSize>,
    }
    @vertex
    fn vsMain(@builtin(vertex_index) vertexIndex : u32) -> VertexOutputs {
          var posAndClipDistances = array(
              vec3f(-1.0,  1.0, -1.0),
              vec3f( 1.0, -1.0,  1.0),
              vec3f( 1.0,  1.0,  0.0),
              vec3f(-1.0, -1.0,  0.0),
              vec3f( 1.0, -1.0,  1.0),
              vec3f(-1.0,  1.0, -1.0));
          var vertexOutput : VertexOutputs;
          vertexOutput.position = vec4f(posAndClipDistances[vertexIndex].xy, 0.0, 1.0);
          vertexOutput.clipDistances[kClipDistancesSize - 1] = posAndClipDistances[vertexIndex].z;
          return vertexOutput;
    }
    @fragment
    fn fsMain() -> @location(0) vec4f {
        return vec4f(1.0, 0.0, 0.0, 1.0);
    }`;
  const module = t.device.createShaderModule({ code });
  const renderPipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module
    },
    fragment: {
      module,
      targets: [
      {
        format: 'rgba8unorm'
      }]

    }
  });

  const kSize = 7;
  const outputTexture = t.createTextureTracked({
    format: 'rgba8unorm',
    size: [kSize, kSize, 1],
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
  });

  // Clear outputTexture to Green
  const commandEncoder = t.device.createCommandEncoder();
  const renderPassEncoder = commandEncoder.beginRenderPass({
    colorAttachments: [
    {
      view: outputTexture.createView(),
      loadOp: 'clear',
      clearValue: { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
      storeOp: 'store'
    }]

  });
  renderPassEncoder.setPipeline(renderPipeline);
  renderPassEncoder.draw(6);
  renderPassEncoder.end();

  const kBytesPerRow = 256;
  const kBytesPerPixel = 4;
  const outputDataSize = kBytesPerRow * (kSize - 1) + kSize * kBytesPerPixel;
  const outputBuffer = t.createBufferTracked({
    size: outputDataSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  commandEncoder.copyTextureToBuffer(
    {
      texture: outputTexture
    },
    {
      buffer: outputBuffer,
      bytesPerRow: kBytesPerRow,
      rowsPerImage: kSize
    },
    [kSize, kSize, 1]
  );
  t.queue.submit([commandEncoder.finish()]);

  // The top-left part should be Green and the bottom-right part should be Red
  const expectedData = new Uint8Array(outputDataSize);
  for (let y = 0; y < kSize; ++y) {
    const baseOffset = kBytesPerRow * y;
    for (let x = 0; x < kSize; ++x) {
      const lastRed = kSize - y - 1;
      for (let i = 0; i < lastRed; ++i) {
        expectedData[baseOffset + i * 4] = 0;
        expectedData[baseOffset + i * 4 + 1] = 255;
        expectedData[baseOffset + i * 4 + 2] = 0;
        expectedData[baseOffset + i * 4 + 3] = 255;
      }
      for (let j = lastRed; j < kSize; ++j) {
        expectedData[baseOffset + j * 4] = 255;
        expectedData[baseOffset + j * 4 + 1] = 0;
        expectedData[baseOffset + j * 4 + 2] = 0;
        expectedData[baseOffset + j * 4 + 3] = 255;
      }
    }
  }
  t.expectGPUBufferValuesEqual(outputBuffer, expectedData);
});