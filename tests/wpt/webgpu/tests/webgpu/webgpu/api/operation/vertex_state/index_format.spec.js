/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test indexing, index format and primitive restart.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';
import { getTextureCopyLayout } from '../../../util/texture/layout.js';

const kHeight = 4;
const kWidth = 8;
const kTextureFormat = 'r8uint';

/** 4x4 grid of r8uint values (each 0 or 1). */







/** Expected 4x4 rasterization of a bottom-left triangle. */
const kBottomLeftTriangle = [
[0, 0, 0, 0, 0, 0, 0, 0],
[0, 0, 0, 0, 1, 0, 0, 0],
[0, 0, 0, 0, 1, 1, 0, 0],
[0, 0, 0, 0, 1, 1, 1, 0]];


/** Expected 4x4 rasterization filling the whole quad. */
const kSquare = [
[0, 0, 0, 0, 1, 1, 1, 1],
[0, 0, 0, 0, 1, 1, 1, 1],
[0, 0, 0, 0, 1, 1, 1, 1],
[0, 0, 0, 0, 1, 1, 1, 1]];


/** Expected 4x4 rasterization with no pixels. */
const kNothing = [
[0, 0, 0, 0, 0, 0, 0, 0],
[0, 0, 0, 0, 0, 0, 0, 0],
[0, 0, 0, 0, 0, 0, 0, 0],
[0, 0, 0, 0, 0, 0, 0, 0]];


const { byteLength, bytesPerRow, rowsPerImage } = getTextureCopyLayout(kTextureFormat, '2d', [
kWidth,
kHeight,
1]
);

class IndexFormatTest extends GPUTest {
  MakeRenderPipeline(
  topology,
  stripIndexFormat)
  {
    const vertexModule = this.device.createShaderModule({
      // NOTE: These positions will create triangles that cut right through pixel centers. If this
      // results in different rasterization results on different hardware, tweak to avoid this.
      code: `
        @vertex
        fn main(@builtin(vertex_index) VertexIndex : u32)
             -> @builtin(position) vec4<f32> {
          var pos = array<vec2<f32>, 4>(
            vec2<f32>(0.01,  0.98),
            vec2<f32>(0.99, -0.98),
            vec2<f32>(0.99,  0.98),
            vec2<f32>(0.01, -0.98));

          if (VertexIndex == 0xFFFFu || VertexIndex == 0xFFFFFFFFu) {
            return vec4<f32>(-0.99, -0.98, 0.0, 1.0);
          }
          return vec4<f32>(pos[VertexIndex], 0.0, 1.0);
        }
      `
    });

    const fragmentModule = this.device.createShaderModule({
      code: `
        @fragment
        fn main() -> @location(0) u32 {
          return 1u;
        }
      `
    });

    return this.device.createRenderPipeline({
      layout: this.device.createPipelineLayout({ bindGroupLayouts: [] }),
      vertex: { module: vertexModule, entryPoint: 'main' },
      fragment: {
        module: fragmentModule,
        entryPoint: 'main',
        targets: [{ format: kTextureFormat }]
      },
      primitive: {
        topology,
        stripIndexFormat
      }
    });
  }

  CreateIndexBuffer(indices, indexFormat) {
    const typedArrayConstructor = { uint16: Uint16Array, uint32: Uint32Array }[indexFormat];
    return this.makeBufferWithContents(new typedArrayConstructor(indices), GPUBufferUsage.INDEX);
  }

  run(
  indexBuffer,
  indexCount,
  indexFormat,
  indexOffset = 0,
  primitiveTopology = 'triangle-list')
  {
    let pipeline;
    // The indexFormat must be set in render pipeline descriptor that specifies a strip primitive
    // topology for primitive restart testing
    if (primitiveTopology === 'line-strip' || primitiveTopology === 'triangle-strip') {
      pipeline = this.MakeRenderPipeline(primitiveTopology, indexFormat);
    } else {
      pipeline = this.MakeRenderPipeline(primitiveTopology);
    }

    const colorAttachment = this.device.createTexture({
      format: kTextureFormat,
      size: { width: kWidth, height: kHeight, depthOrArrayLayers: 1 },
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
    });

    const result = this.device.createBuffer({
      size: byteLength,
      usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
    });

    const encoder = this.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: colorAttachment.createView(),
        clearValue: [0, 0, 0, 0],
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });
    pass.setPipeline(pipeline);
    pass.setIndexBuffer(indexBuffer, indexFormat, indexOffset);
    pass.drawIndexed(indexCount);
    pass.end();
    encoder.copyTextureToBuffer(
      { texture: colorAttachment },
      { buffer: result, bytesPerRow, rowsPerImage },
      [kWidth, kHeight]
    );
    this.device.queue.submit([encoder.finish()]);

    return result;
  }

  CreateExpectedUint8Array(renderShape) {
    const arrayBuffer = new Uint8Array(byteLength);
    for (let row = 0; row < renderShape.length; row++) {
      for (let col = 0; col < renderShape[row].length; col++) {
        const texel = renderShape[row][col];

        const kBytesPerTexel = 1; // r8uint
        const byteOffset = row * bytesPerRow + col * kBytesPerTexel;
        arrayBuffer[byteOffset] = texel;
      }
    }
    return arrayBuffer;
  }
}

export const g = makeTestGroup(IndexFormatTest);

g.test('index_format,uint16').
desc('Test rendering result of indexed draw with index format of uint16.').
paramsSubcasesOnly([
{ indexOffset: 0, _indexCount: 10, _expectedShape: kSquare },
{ indexOffset: 6, _indexCount: 6, _expectedShape: kBottomLeftTriangle },
{ indexOffset: 18, _indexCount: 0, _expectedShape: kNothing }]
).
fn((t) => {
  const { indexOffset, _indexCount, _expectedShape } = t.params;

  // If this is written as uint16 but interpreted as uint32, it will have index 1 and 2 be both 0
  // and render nothing.
  // And the index buffer size - offset must be not less than the size required by triangle
  // list, otherwise it also render nothing.
  const indices = [1, 2, 0, 0, 0, 0, 0, 1, 3, 0];
  const indexBuffer = t.CreateIndexBuffer(indices, 'uint16');
  const result = t.run(indexBuffer, _indexCount, 'uint16', indexOffset);

  const expectedTextureValues = t.CreateExpectedUint8Array(_expectedShape);
  t.expectGPUBufferValuesEqual(result, expectedTextureValues);
});

g.test('index_format,uint32').
desc('Test rendering result of indexed draw with index format of uint32.').
paramsSubcasesOnly([
{ indexOffset: 0, _indexCount: 10, _expectedShape: kSquare },
{ indexOffset: 12, _indexCount: 7, _expectedShape: kBottomLeftTriangle },
{ indexOffset: 36, _indexCount: 0, _expectedShape: kNothing }]
).
fn((t) => {
  const { indexOffset, _indexCount, _expectedShape } = t.params;

  // If this is interpreted as uint16, then it would be 0, 1, 0, ... and would draw nothing.
  // And the index buffer size - offset must be not less than the size required by triangle
  // list, otherwise it also render nothing.
  const indices = [1, 2, 0, 0, 0, 0, 0, 1, 3, 0];
  const indexBuffer = t.CreateIndexBuffer(indices, 'uint32');
  const result = t.run(indexBuffer, _indexCount, 'uint32', indexOffset);

  const expectedTextureValues = t.CreateExpectedUint8Array(_expectedShape);
  t.expectGPUBufferValuesEqual(result, expectedTextureValues);
});

g.test('index_format,change_pipeline_after_setIndexBuffer').
desc('Test that setting the index buffer before the pipeline works correctly.').
params((u) => u.combine('setPipelineBeforeSetIndexBuffer', [false, true])).
fn((t) => {
  const indexOffset = 12;
  const indexCount = 7;
  const expectedShape = kBottomLeftTriangle;

  const indexFormat16 = 'uint16';
  const indexFormat32 = 'uint32';

  const indices = [1, 2, 0, 0, 0, 0, 0, 1, 3, 0];
  const indexBuffer = t.CreateIndexBuffer(indices, indexFormat32);

  const kPrimitiveTopology = 'triangle-strip';
  const pipeline32 = t.MakeRenderPipeline(kPrimitiveTopology, indexFormat32);
  const pipeline16 = t.MakeRenderPipeline(kPrimitiveTopology, indexFormat16);

  const colorAttachment = t.device.createTexture({
    format: kTextureFormat,
    size: { width: kWidth, height: kHeight, depthOrArrayLayers: 1 },
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const result = t.device.createBuffer({
    size: byteLength,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: colorAttachment.createView(),
      clearValue: [0, 0, 0, 0],
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });

  if (t.params.setPipelineBeforeSetIndexBuffer) {
    pass.setPipeline(pipeline16);
  }
  pass.setIndexBuffer(indexBuffer, indexFormat32, indexOffset);
  pass.setPipeline(pipeline32); // Set the pipeline for 'indexFormat32' again.
  pass.drawIndexed(indexCount);
  pass.end();
  encoder.copyTextureToBuffer(
    { texture: colorAttachment },
    { buffer: result, bytesPerRow, rowsPerImage },
    [kWidth, kHeight]
  );
  t.device.queue.submit([encoder.finish()]);

  const expectedTextureValues = t.CreateExpectedUint8Array(expectedShape);
  t.expectGPUBufferValuesEqual(result, expectedTextureValues);
});

g.test('index_format,setIndexBuffer_before_setPipeline').
desc('Test that setting the index buffer before the pipeline works correctly.').
params((u) => u.combine('setIndexBufferBeforeSetPipeline', [false, true])).
fn((t) => {
  const indexOffset = 12;
  const indexCount = 7;
  const expectedShape = kBottomLeftTriangle;

  const indexFormat = 'uint32';

  const indices = [1, 2, 0, 0, 0, 0, 0, 1, 3, 0];
  const indexBuffer = t.CreateIndexBuffer(indices, indexFormat);

  const kPrimitiveTopology = 'triangle-strip';
  const pipeline = t.MakeRenderPipeline(kPrimitiveTopology, indexFormat);

  const colorAttachment = t.device.createTexture({
    format: kTextureFormat,
    size: { width: kWidth, height: kHeight, depthOrArrayLayers: 1 },
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const result = t.device.createBuffer({
    size: byteLength,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: colorAttachment.createView(),
      clearValue: [0, 0, 0, 0],
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });

  if (t.params.setIndexBufferBeforeSetPipeline) {
    pass.setIndexBuffer(indexBuffer, indexFormat, indexOffset);
    pass.setPipeline(pipeline);
  } else {
    pass.setPipeline(pipeline);
    pass.setIndexBuffer(indexBuffer, indexFormat, indexOffset);
  }

  pass.drawIndexed(indexCount);
  pass.end();
  encoder.copyTextureToBuffer(
    { texture: colorAttachment },
    { buffer: result, bytesPerRow, rowsPerImage },
    [kWidth, kHeight]
  );
  t.device.queue.submit([encoder.finish()]);

  const expectedTextureValues = t.CreateExpectedUint8Array(expectedShape);
  t.expectGPUBufferValuesEqual(result, expectedTextureValues);
});

g.test('index_format,setIndexBuffer_different_formats').
desc(
  `
  Test that index buffers of multiple formats can be used with a pipeline that doesn't use strip
  primitive topology.
  `
).
fn((t) => {
  const indices = [1, 2, 0, 0, 0, 0, 0, 1, 3, 0];

  // Create a pipeline to be used by different index formats.
  const kPrimitiveTopology = 'triangle-list';
  const pipeline = t.MakeRenderPipeline(kPrimitiveTopology);

  const expectedTextureValues = t.CreateExpectedUint8Array(kBottomLeftTriangle);

  const colorAttachment = t.device.createTexture({
    format: kTextureFormat,
    size: { width: kWidth, height: kHeight, depthOrArrayLayers: 1 },
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const result = t.device.createBuffer({
    size: byteLength,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  let encoder = t.device.createCommandEncoder();
  {
    const indexFormat = 'uint32';
    const indexOffset = 12;
    const indexCount = 7;
    const indexBuffer = t.CreateIndexBuffer(indices, indexFormat);

    const pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: colorAttachment.createView(),
        clearValue: [0, 0, 0, 0],
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });

    pass.setIndexBuffer(indexBuffer, indexFormat, indexOffset);
    pass.setPipeline(pipeline);
    pass.drawIndexed(indexCount);
    pass.end();
    encoder.copyTextureToBuffer(
      { texture: colorAttachment },
      { buffer: result, bytesPerRow, rowsPerImage },
      [kWidth, kHeight]
    );
  }
  t.device.queue.submit([encoder.finish()]);
  t.expectGPUBufferValuesEqual(result, expectedTextureValues);

  // Call setIndexBuffer with the pipeline and a different index format buffer.
  encoder = t.device.createCommandEncoder();
  {
    const indexFormat = 'uint16';
    const indexOffset = 6;
    const indexCount = 6;
    const indexBuffer = t.CreateIndexBuffer(indices, indexFormat);

    const pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: colorAttachment.createView(),
        clearValue: [0, 0, 0, 0],
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });

    pass.setIndexBuffer(indexBuffer, indexFormat, indexOffset);
    pass.setPipeline(pipeline);
    pass.drawIndexed(indexCount);
    pass.end();
    encoder.copyTextureToBuffer(
      { texture: colorAttachment },
      { buffer: result, bytesPerRow, rowsPerImage },
      [kWidth, kHeight]
    );
  }
  t.device.queue.submit([encoder.finish()]);
  t.expectGPUBufferValuesEqual(result, expectedTextureValues);
});

g.test('primitive_restart').
desc(
  `
Test primitive restart with each primitive topology.

Primitive restart should be always active with strip primitive topologies
('line-strip' or 'triangle-strip') and never active for other topologies, where
the primitive restart value isn't special and should be treated as a regular index value.

The value -1 gets uploaded as 0xFFFF or 0xFFFF_FFFF according to the format.

The positions of these points are embedded in the shader above, and look like this:
  |   0  2|
  |       |
  -1  3  1|

Below are the indices lists used for each test, and the expected rendering result of each
(approximately, in the case of incorrect results). This shows the expected result (marked '->')
is different from what you would get if the topology were incorrect.

- primitiveTopology: triangle-list
  indices: [0, 1, 3, -1, 2, 1, 0, 0],
   -> triangle-list:              (0, 1, 3), (-1, 2, 1)
        |    #  #|
        |    ####|
        |   #####|
        | #######|
      triangle-list with restart: (0, 1, 3), (2, 1, 0)
      triangle-strip:             (0, 1, 3), (2, 1, 0), (1, 0, 0)
        |    ####|
        |    ####|
        |    ####|
        |    ####|
      triangle-strip w/o restart: (0, 1, 3), (1, 3, -1), (3, -1, 2), (-1, 2, 1), (2, 1, 0), (1, 0, 0)
        |    ####|
        |    ####|
        |   #####|
        | #######|

- primitiveTopology: triangle-strip
  indices: [3, 1, 0, -1, 2, 2, 1, 3],
   -> triangle-strip:             (3, 1, 0), (2, 2, 1), (2, 1, 3)
        |    #  #|
        |    ####|
        |    ####|
        |    ####|
      triangle-strip w/o restart: (3, 1, 0), (1, 0, -1), (0, -1, 2), (2, 2, 1), (2, 3, 1)
        |    ####|
        |   #####|
        |  ######|
        | #######|
      triangle-list:              (3, 1, 0), (-1, 2, 2)
      triangle-list with restart: (3, 1, 0), (2, 2, 1)
        |        |
        |    #   |
        |    ##  |
        |    ### |

- primitiveTopology: point, line-list, line-strip:
  indices: [0, 1, -1, 2, -1, 2, 3, 0],
   -> point-list:             (0), (1), (-1), (2), (3), (0)
        |    #  #|
        |        |
        |        |
        |#   #  #|
      point-list with restart (0), (1), (2), (3), (0)
        |    #  #|
        |        |
        |        |
        |    #  #|
   -> line-list:              (0, 1), (-1, 2), (3, 0)
        |    # ##|
        |    ##  |
        |  ### # |
        |##  #  #|
      line-list with restart: (0, 1), (2, 3)
        |    #  #|
        |     ## |
        |     ## |
        |    #  #|
   -> line-strip:             (0, 1), (2, 3), (3, 0)
        |    #  #|
        |    ### |
        |    ### |
        |    #  #|
      line-strip w/o restart: (0, 1), (1, -1), (-1, 2), (2, 3), (3, 3)
        |    # ##|
        |    ### |
        |  ## ## |
        |########|
`
).
params((u) =>
u //
.combine('indexFormat', ['uint16', 'uint32']).
combineWithParams([
{
  primitiveTopology: 'point-list',
  _indices: [0, 1, -1, 2, 3, 0],
  _expectedShape: [
  [0, 0, 0, 0, 1, 0, 0, 1],
  [0, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0, 0, 0, 0],
  [1, 0, 0, 0, 1, 0, 0, 1]]

},
{
  primitiveTopology: 'line-list',
  _indices: [0, 1, -1, 2, 3, 0],
  _expectedShape: [
  [0, 0, 0, 0, 1, 0, 1, 1],
  [0, 0, 0, 0, 1, 1, 0, 0],
  [0, 0, 1, 1, 1, 0, 1, 0],
  [1, 1, 0, 0, 1, 0, 0, 1]]

},
{
  primitiveTopology: 'line-strip',
  _indices: [0, 1, -1, 2, 3, 0],
  _expectedShape: [
  [0, 0, 0, 0, 1, 0, 0, 1],
  [0, 0, 0, 0, 1, 1, 1, 0],
  [0, 0, 0, 0, 1, 1, 1, 0],
  [0, 0, 0, 0, 1, 0, 0, 1]]

},
{
  primitiveTopology: 'triangle-list',
  _indices: [0, 1, 3, -1, 2, 1, 0, 0],
  _expectedShape: [
  [0, 0, 0, 0, 0, 0, 0, 1],
  [0, 0, 0, 0, 1, 1, 1, 1],
  [0, 0, 0, 1, 1, 1, 1, 1],
  [0, 1, 1, 1, 1, 1, 1, 1]]

},
{
  primitiveTopology: 'triangle-strip',
  _indices: [3, 1, 0, -1, 2, 2, 1, 3],
  _expectedShape: [
  [0, 0, 0, 0, 0, 0, 0, 1],
  [0, 0, 0, 0, 1, 0, 1, 1],
  [0, 0, 0, 0, 1, 1, 1, 1],
  [0, 0, 0, 0, 1, 1, 1, 1]]

}]
)
).
fn((t) => {
  const { indexFormat, primitiveTopology, _indices, _expectedShape } = t.params;

  const indexBuffer = t.CreateIndexBuffer(_indices, indexFormat);
  const result = t.run(indexBuffer, _indices.length, indexFormat, 0, primitiveTopology);

  const expectedTextureValues = t.CreateExpectedUint8Array(_expectedShape);
  t.expectGPUBufferValuesEqual(result, expectedTextureValues);
});