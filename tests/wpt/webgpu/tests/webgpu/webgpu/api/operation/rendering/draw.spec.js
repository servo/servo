/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for the general aspects of draw/drawIndexed/drawIndirect/drawIndexedIndirect.

Primitive topology tested in api/operation/render_pipeline/primitive_topology.spec.ts.
Index format tested in api/operation/command_buffer/render/state_tracking.spec.ts.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import {
  assert } from


'../../../../common/util/util.js';
import { GPUTest, TextureTestMixin } from '../../../gpu_test.js';


class DrawTest extends TextureTestMixin(GPUTest) {
  checkTriangleDraw(opts)









  {
    // Set fallbacks when parameters are undefined in order to calculate the expected values.
    const defaulted = {
      firstIndex: opts.firstIndex ?? 0,
      count: opts.count,
      firstInstance: opts.firstInstance ?? 0,
      instanceCount: opts.instanceCount ?? 1,
      indexed: opts.indexed,
      indirect: opts.indirect,
      vertexBufferOffset: opts.vertexBufferOffset,
      indexBufferOffset: opts.indexBufferOffset ?? 0,
      baseVertex: opts.baseVertex ?? 0
    };

    const renderTargetSize = [72, 36];

    // The test will split up the render target into a grid where triangles of
    // increasing primitive id will be placed along the X axis, and triangles
    // of increasing instance id will be placed along the Y axis. The size of the
    // grid is based on the max primitive id and instance id used.
    const numX = 6;
    const numY = 6;
    const tileSizeX = renderTargetSize[0] / numX;
    const tileSizeY = renderTargetSize[1] / numY;

    // |\
    // |   \
    // |______\
    // Unit triangle shaped like this. 0-1 Y-down.

    const triangleVertices = [
    0.0, 0.0,
    0.0, 1.0,
    1.0, 1.0];


    const renderTarget = this.createTextureTracked({
      size: renderTargetSize,
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
      format: 'rgba8unorm'
    });

    const vertexModule = this.device.createShaderModule({
      code: `
struct Inputs {
  @builtin(vertex_index) vertex_index : u32,
  @builtin(instance_index) instance_id : u32,
  @location(0) vertexPosition : vec2<f32>,
};

@vertex fn vert_main(input : Inputs
  ) -> @builtin(position) vec4<f32> {
  // 3u is the number of points in a triangle to convert from index
  // to id.
  var vertex_id : u32 = input.vertex_index / 3u;

  var x : f32 = (input.vertexPosition.x + f32(vertex_id)) / ${numX}.0;
  var y : f32 = (input.vertexPosition.y + f32(input.instance_id)) / ${numY}.0;

  // (0,1) y-down space to (-1,1) y-up NDC
  x = 2.0 * x - 1.0;
  y = -2.0 * y + 1.0;
  return vec4<f32>(x, y, 0.0, 1.0);
}
`
    });

    const fragmentModule = this.device.createShaderModule({
      code: `
struct Output {
  value : u32
};

@group(0) @binding(0) var<storage, read_write> output : Output;

@fragment fn frag_main() -> @location(0) vec4<f32> {
  output.value = 1u;
  return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}
`
    });

    const pipeline = this.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: vertexModule,
        entryPoint: 'vert_main',
        buffers: [
        {
          attributes: [
          {
            shaderLocation: 0,
            format: 'float32x2',
            offset: 0
          }],

          arrayStride: 2 * Float32Array.BYTES_PER_ELEMENT
        }]

      },
      fragment: {
        module: fragmentModule,
        entryPoint: 'frag_main',
        targets: [
        {
          format: 'rgba8unorm'
        }]

      }
    });

    const resultBuffer = this.createBufferTracked({
      size: Uint32Array.BYTES_PER_ELEMENT,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
    });

    const resultBindGroup = this.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
      {
        binding: 0,
        resource: {
          buffer: resultBuffer
        }
      }]

    });

    const commandEncoder = this.device.createCommandEncoder();
    const renderPass = commandEncoder.beginRenderPass({
      colorAttachments: [
      {
        view: renderTarget.createView(),
        clearValue: [0, 0, 0, 0],
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });

    renderPass.setPipeline(pipeline);
    renderPass.setBindGroup(0, resultBindGroup);

    if (defaulted.indexed) {
      // INDEXED DRAW
      assert(defaulted.baseVertex !== undefined);
      assert(defaulted.indexBufferOffset !== undefined);

      renderPass.setIndexBuffer(
        this.makeBufferWithContents(
          new Uint32Array([
          // Offset the index buffer contents by empty data.
          ...new Array(defaulted.indexBufferOffset / Uint32Array.BYTES_PER_ELEMENT),

          0, 1, 2, //
          3, 4, 5, //
          6, 7, 8 //
          ]),
          GPUBufferUsage.INDEX
        ),
        'uint32',
        defaulted.indexBufferOffset
      );

      renderPass.setVertexBuffer(
        0,
        this.makeBufferWithContents(
          new Float32Array([
          // Offset the vertex buffer contents by empty data.
          ...new Array(defaulted.vertexBufferOffset / Float32Array.BYTES_PER_ELEMENT),

          // selected with base_vertex=0
          // count=6
          ...triangleVertices, //   |   count=6;first=3
          ...triangleVertices, //   |       |
          ...triangleVertices, //           |

          // selected with base_vertex=9
          // count=6
          ...triangleVertices, //   |   count=6;first=3
          ...triangleVertices, //   |       |
          ...triangleVertices //           |
          ]),
          GPUBufferUsage.VERTEX
        ),
        defaulted.vertexBufferOffset
      );

      if (defaulted.indirect) {
        const args = [
        defaulted.count,
        defaulted.instanceCount,
        defaulted.firstIndex,
        defaulted.baseVertex,
        defaulted.firstInstance];

        renderPass.drawIndexedIndirect(
          this.makeBufferWithContents(new Uint32Array(args), GPUBufferUsage.INDIRECT),
          0
        );
      } else {
        const args = [
        opts.count,
        opts.instanceCount,
        opts.firstIndex,
        opts.baseVertex,
        opts.firstInstance];

        renderPass.drawIndexed.apply(renderPass, [...args]);
      }
    } else {
      // NON-INDEXED DRAW
      renderPass.setVertexBuffer(
        0,
        this.makeBufferWithContents(
          new Float32Array([
          // Offset the vertex buffer contents by empty data.
          ...new Array(defaulted.vertexBufferOffset / Float32Array.BYTES_PER_ELEMENT),

          // count=6
          ...triangleVertices, //   |   count=6;first=3
          ...triangleVertices, //   |       |
          ...triangleVertices //           |
          ]),
          GPUBufferUsage.VERTEX
        ),
        defaulted.vertexBufferOffset
      );

      if (defaulted.indirect) {
        const args = [
        defaulted.count,
        defaulted.instanceCount,
        defaulted.firstIndex,
        defaulted.firstInstance];

        renderPass.drawIndirect(
          this.makeBufferWithContents(new Uint32Array(args), GPUBufferUsage.INDIRECT),
          0
        );
      } else {
        const args = [opts.count, opts.instanceCount, opts.firstIndex, opts.firstInstance];
        renderPass.draw.apply(renderPass, [...args]);
      }
    }

    renderPass.end();
    this.queue.submit([commandEncoder.finish()]);

    const green = new Uint8Array([0, 255, 0, 255]);
    const transparentBlack = new Uint8Array([0, 0, 0, 0]);

    const didDraw = defaulted.count && defaulted.instanceCount;

    this.expectGPUBufferValuesEqual(resultBuffer, new Uint32Array([didDraw ? 1 : 0]));

    const baseVertexCount = defaulted.baseVertex ?? 0;
    const pixelComparisons = [];
    for (let primitiveId = 0; primitiveId < numX; ++primitiveId) {
      for (let instanceId = 0; instanceId < numY; ++instanceId) {
        let expectedColor = didDraw ? green : transparentBlack;
        if (
        primitiveId * 3 < defaulted.firstIndex + baseVertexCount ||
        primitiveId * 3 >= defaulted.firstIndex + baseVertexCount + defaulted.count)
        {
          expectedColor = transparentBlack;
        }

        if (
        instanceId < defaulted.firstInstance ||
        instanceId >= defaulted.firstInstance + defaulted.instanceCount)
        {
          expectedColor = transparentBlack;
        }

        pixelComparisons.push({
          coord: { x: (1 / 3 + primitiveId) * tileSizeX, y: (2 / 3 + instanceId) * tileSizeY },
          exp: expectedColor
        });
      }
    }
    this.expectSinglePixelComparisonsAreOkInTexture({ texture: renderTarget }, pixelComparisons);
  }
}

export const g = makeTestGroup(DrawTest);

g.test('arguments').
desc(
  `Test that draw arguments are passed correctly by drawing triangles in a grid.
Horizontally across the texture are triangles with increasing "primitive id".
Vertically down the screen are triangles with increasing instance id.
Increasing the |first| param should skip some of the beginning triangles on the horizontal axis.
Increasing the |first_instance| param should skip of the beginning triangles on the vertical axis.
The vertex buffer contains two sets of disjoint triangles, and base_vertex is used to select the second set.
The test checks that the center of all of the expected triangles is drawn, and the others are empty.
The fragment shader also writes out to a storage buffer. If the draw is zero-sized, check that no value is written.

Params:
  - first= {0, 3} - either the firstVertex or firstIndex
  - count= {0, 3, 6} - either the vertexCount or indexCount
  - first_instance= {0, 2}
  - instance_count= {0, 1, 4}
  - indexed= {true, false}
  - indirect= {true, false}
  - vertex_buffer_offset= {0, 32}
  - index_buffer_offset= {0, 16} - only for indexed draws
  - base_vertex= {0, 9} - only for indexed draws
  `
).
params((u) =>
u.
combine('first', [0, 3]).
combine('count', [0, 3, 6]).
combine('first_instance', [0, 2]).
combine('instance_count', [0, 1, 4]).
combine('indexed', [false, true]).
combine('indirect', [false, true]).
combine('vertex_buffer_offset', [0, 32]).
expand('index_buffer_offset', (p) => p.indexed ? [0, 16] : [undefined]).
expand('base_vertex', (p) => p.indexed ? [0, 9] : [undefined])
).
beforeAllSubcases((t) => {
  if (t.params.first_instance > 0 && t.params.indirect) {
    t.selectDeviceOrSkipTestCase('indirect-first-instance');
  }
}).
fn((t) => {
  t.checkTriangleDraw({
    firstIndex: t.params.first,
    count: t.params.count,
    firstInstance: t.params.first_instance,
    instanceCount: t.params.instance_count,
    indexed: t.params.indexed,
    indirect: t.params.indirect,
    vertexBufferOffset: t.params.vertex_buffer_offset,
    indexBufferOffset: t.params.index_buffer_offset,
    baseVertex: t.params.base_vertex
  });
});

g.test('default_arguments').
desc(
  `
  Test that defaults arguments are passed correctly by drawing triangles in a grid when they are not
  defined. This test is written based on the 'arguments' with 'undefined' value in the parameters.
    - mode= {draw, drawIndexed}
    - arg= {instance_count, first_index, first_instance, base_vertex}
  `
).
params((u) =>
u.
combine('mode', ['draw', 'drawIndexed']).
beginSubcases().
combine('instance_count', [undefined, 4]).
combine('first_index', [undefined, 3]).
combine('first_instance', [undefined, 2]).
expand('base_vertex', (p) =>
p.mode === 'drawIndexed' ? [undefined, 9] : [undefined]
)
).
fn((t) => {
  const kVertexCount = 3;
  const kVertexBufferOffset = 32;
  const kIndexBufferOffset = 16;

  t.checkTriangleDraw({
    firstIndex: t.params.first_index,
    count: kVertexCount,
    firstInstance: t.params.first_instance,
    instanceCount: t.params.instance_count,
    indexed: t.params.mode === 'drawIndexed',
    indirect: false, // indirect
    vertexBufferOffset: kVertexBufferOffset,
    indexBufferOffset: kIndexBufferOffset,
    baseVertex: t.params.base_vertex
  });
});

g.test('vertex_attributes,basic').
desc(
  `Test basic fetching of vertex attributes.
  Each vertex attribute is a single value and written out into a storage buffer.
  Tests that vertices with offsets/strides for instanced/non-instanced attributes are
  fetched correctly. Not all vertex formats are tested.

  Params:
  - vertex_attribute_count= {1, 4, 8, 16}
  - vertex_buffer_count={1, 4, 8} - where # attributes is > 0
  - vertex_format={uint32, float32}
  - step_mode= {undefined, vertex, instance, mixed} - where mixed only applies for vertex_buffer_count > 1
  `
).
params((u) =>
u.
combine('vertex_attribute_count', [1, 4, 8, 16]).
combine('vertex_buffer_count', [1, 4, 8]).
combine('vertex_format', ['uint32', 'float32']).
combine('step_mode', [undefined, 'vertex', 'instance', 'mixed']).
unless((p) => p.vertex_attribute_count < p.vertex_buffer_count).
unless((p) => p.step_mode === 'mixed' && p.vertex_buffer_count <= 1)
).
fn((t) => {
  const vertexCount = 4;
  const instanceCount = 4;

  // In compat mode, @builtin(vertex_index) and @builtin(instance_index) each take an attribute.
  const maxAttributes = t.device.limits.maxVertexAttributes - (t.isCompatibility ? 2 : 0);
  const numAttributes = Math.min(maxAttributes, t.params.vertex_attribute_count);
  const maxAttributesPerVertexBuffer = Math.ceil(numAttributes / t.params.vertex_buffer_count);

  let shaderLocation = 0;
  let attributeValue = 0;
  const bufferLayouts = [];

  let ExpectedDataConstructor;
  switch (t.params.vertex_format) {
    case 'uint32':
      ExpectedDataConstructor = Uint32Array;
      break;
    case 'float32':
      ExpectedDataConstructor = Float32Array;
      break;
  }

  // Populate |bufferLayouts|, |vertexBufferData|, and |vertexBuffers|.
  // We will use this to both create the render pipeline, and produce the
  // expected data on the CPU.
  // Attributes in each buffer will be interleaved.
  const vertexBuffers = [];
  const vertexBufferData = [];
  for (let b = 0; b < t.params.vertex_buffer_count; ++b) {
    const vertexBufferValues = [];

    let offset = 0;
    let stepMode = t.params.step_mode;

    // If stepMode is mixed, alternate between vertex and instance.
    if (stepMode === 'mixed') {
      stepMode = ['vertex', 'instance'][b % 2];
    }

    let vertexOrInstanceCount;
    switch (stepMode) {
      case undefined:
      case 'vertex':
        vertexOrInstanceCount = vertexCount;
        break;
      case 'instance':
        vertexOrInstanceCount = instanceCount;
        break;
    }

    const attributes = [];
    const numAttributesForBuffer = Math.min(
      maxAttributesPerVertexBuffer,
      maxAttributes - b * maxAttributesPerVertexBuffer
    );

    for (let a = 0; a < numAttributesForBuffer; ++a) {
      const attribute = {
        format: t.params.vertex_format,
        shaderLocation,
        offset
      };
      attributes.push(attribute);

      offset += ExpectedDataConstructor.BYTES_PER_ELEMENT;
      shaderLocation += 1;
    }

    for (let v = 0; v < vertexOrInstanceCount; ++v) {
      for (let a = 0; a < numAttributesForBuffer; ++a) {
        vertexBufferValues.push(attributeValue);
        attributeValue += 1.234; // Values will get rounded later if we make a Uint32Array.
      }
    }

    bufferLayouts.push({
      attributes,
      arrayStride: offset,
      stepMode
    });

    const data = new ExpectedDataConstructor(vertexBufferValues);
    vertexBufferData.push(data);
    vertexBuffers.push(t.makeBufferWithContents(data, GPUBufferUsage.VERTEX));
  }

  // Create an array of shader locations [0, 1, 2, 3, ...] for easy iteration.
  const vertexInputShaderLocations = new Array(shaderLocation).fill(0).map((_, i) => i);

  // Create the expected data buffer.
  const expectedData = new ExpectedDataConstructor(
    vertexCount * instanceCount * vertexInputShaderLocations.length
  );

  // Populate the expected data. This is a CPU-side version of what we expect the shader
  // to do.
  for (let vertexIndex = 0; vertexIndex < vertexCount; ++vertexIndex) {
    for (let instanceIndex = 0; instanceIndex < instanceCount; ++instanceIndex) {
      bufferLayouts.forEach((bufferLayout, b) => {
        for (const attribute of bufferLayout.attributes) {
          const primitiveId = vertexCount * instanceIndex + vertexIndex;
          const outputIndex =
          primitiveId * vertexInputShaderLocations.length + attribute.shaderLocation;

          let vertexOrInstanceIndex;
          switch (bufferLayout.stepMode) {
            case undefined:
            case 'vertex':
              vertexOrInstanceIndex = vertexIndex;
              break;
            case 'instance':
              vertexOrInstanceIndex = instanceIndex;
              break;
          }

          const view = new ExpectedDataConstructor(
            vertexBufferData[b].buffer,
            bufferLayout.arrayStride * vertexOrInstanceIndex + attribute.offset,
            1
          );
          expectedData[outputIndex] = view[0];
        }
      });
    }
  }

  let wgslFormat;
  switch (t.params.vertex_format) {
    case 'uint32':
      wgslFormat = 'u32';
      break;
    case 'float32':
      wgslFormat = 'f32';
      break;
  }

  // Maximum inter-stage shader location is 14, and we need to consume one for primitiveId, 12 for
  // location 0 to 11,  and combine the remaining vertex inputs into one location (one
  // vec4<wgslFormat> when vertex_attribute_count === 16).
  const interStageScalarShaderLocation = Math.min(shaderLocation, 12);
  const interStageScalarShaderLocations = new Array(interStageScalarShaderLocation).
  fill(0).
  map((_, i) => i);

  let accumulateVariableDeclarationsInVertexShader = '';
  let accumulateVariableAssignmentsInVertexShader = '';
  let accumulateVariableDeclarationsInFragmentShader = '';
  let accumulateVariableAssignmentsInFragmentShader = '';
  // The remaining 3 vertex attributes
  if (numAttributes === 16) {
    accumulateVariableDeclarationsInVertexShader = `
        @location(13) @interpolate(flat, either) outAttrib13 : vec4<${wgslFormat}>,
      `;
    accumulateVariableAssignmentsInVertexShader = `
      output.outAttrib13 =
          vec4<${wgslFormat}>(input.attrib12, input.attrib13, input.attrib14, input.attrib15);
      `;
    accumulateVariableDeclarationsInFragmentShader = `
      @location(13) @interpolate(flat, either) attrib13 : vec4<${wgslFormat}>,
      `;
    accumulateVariableAssignmentsInFragmentShader = `
      outBuffer.primitives[input.primitiveId].attrib12 = input.attrib13.x;
      outBuffer.primitives[input.primitiveId].attrib13 = input.attrib13.y;
      outBuffer.primitives[input.primitiveId].attrib14 = input.attrib13.z;
      outBuffer.primitives[input.primitiveId].attrib15 = input.attrib13.w;
      `;
  } else if (numAttributes === 14) {
    accumulateVariableDeclarationsInVertexShader = `
        @location(13) @interpolate(flat, either) outAttrib13 : vec4<${wgslFormat}>,
      `;
    accumulateVariableAssignmentsInVertexShader = `
      output.outAttrib13 =
          vec4<${wgslFormat}>(input.attrib12, input.attrib13, 0, 0);
      `;
    accumulateVariableDeclarationsInFragmentShader = `
      @location(13) @interpolate(flat, either) attrib13 : vec4<${wgslFormat}>,
      `;
    accumulateVariableAssignmentsInFragmentShader = `
      outBuffer.primitives[input.primitiveId].attrib12 = input.attrib13.x;
      outBuffer.primitives[input.primitiveId].attrib13 = input.attrib13.y;
      `;
  }

  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code: `
struct Inputs {
  @builtin(vertex_index) vertexIndex : u32,
  @builtin(instance_index) instanceIndex : u32,
${vertexInputShaderLocations.map((i) => `  @location(${i}) attrib${i} : ${wgslFormat},`).join('\n')}
};

struct Outputs {
  @builtin(position) Position : vec4<f32>,
${interStageScalarShaderLocations.
        map((i) => `  @location(${i}) @interpolate(flat, either) outAttrib${i} : ${wgslFormat},`).
        join('\n')}
  @location(${interStageScalarShaderLocations.length}) @interpolate(flat, either) primitiveId : u32,
${accumulateVariableDeclarationsInVertexShader}
};

@vertex fn main(input : Inputs) -> Outputs {
  var output : Outputs;
${interStageScalarShaderLocations.map((i) => `  output.outAttrib${i} = input.attrib${i};`).join('\n')}
${accumulateVariableAssignmentsInVertexShader}

  output.primitiveId = input.instanceIndex * ${instanceCount}u + input.vertexIndex;
  output.Position = vec4<f32>(0.0, 0.0, 0.5, 1.0);
  return output;
}
          `
      }),
      entryPoint: 'main',
      buffers: bufferLayouts
    },
    fragment: {
      module: t.device.createShaderModule({
        code: `
struct Inputs {
${interStageScalarShaderLocations.
        map((i) => `  @location(${i}) @interpolate(flat, either) attrib${i} : ${wgslFormat},`).
        join('\n')}
  @location(${interStageScalarShaderLocations.length}) @interpolate(flat, either) primitiveId : u32,
${accumulateVariableDeclarationsInFragmentShader}
};

struct OutPrimitive {
${vertexInputShaderLocations.map((i) => `  attrib${i} : ${wgslFormat},`).join('\n')}
};
struct OutBuffer {
  primitives : array<OutPrimitive>
};
@group(0) @binding(0) var<storage, read_write> outBuffer : OutBuffer;

@fragment fn main(input : Inputs) {
${interStageScalarShaderLocations.
        map((i) => `  outBuffer.primitives[input.primitiveId].attrib${i} = input.attrib${i};`).
        join('\n')}
${accumulateVariableAssignmentsInFragmentShader}
}
          `
      }),
      entryPoint: 'main',
      targets: [
      {
        format: 'rgba8unorm',
        writeMask: 0
      }]

    },
    primitive: {
      topology: 'point-list'
    }
  });

  const resultBuffer = t.createBufferTracked({
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    size: vertexCount * instanceCount * vertexInputShaderLocations.length * 4
  });

  const resultBindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: {
        buffer: resultBuffer
      }
    }]

  });

  const commandEncoder = t.device.createCommandEncoder();
  const renderPass = commandEncoder.beginRenderPass({
    colorAttachments: [
    {
      // Dummy render attachment - not used (WebGPU doesn't allow using a render pass with no
      // attachments)
      view: t.
      createTextureTracked({
        usage: GPUTextureUsage.RENDER_ATTACHMENT,
        size: [1],
        format: 'rgba8unorm'
      }).
      createView(),
      clearValue: [0, 0, 0, 0],
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });

  renderPass.setPipeline(pipeline);
  renderPass.setBindGroup(0, resultBindGroup);
  for (let i = 0; i < t.params.vertex_buffer_count; ++i) {
    renderPass.setVertexBuffer(i, vertexBuffers[i]);
  }
  renderPass.draw(vertexCount, instanceCount);
  renderPass.end();
  t.device.queue.submit([commandEncoder.finish()]);

  t.expectGPUBufferValuesEqual(resultBuffer, expectedData);
});

g.test('vertex_attributes,formats').
desc(
  `Test all vertex formats are fetched correctly.

    Runs a basic vertex shader which loads vertex data from two attributes which
    may have different formats. Write data out to a storage buffer and check that
    it was loaded correctly.

    Params:
      - vertex_format_1={...all_vertex_formats}
      - vertex_format_2={...all_vertex_formats}
  `
).
unimplemented();

g.test(`largeish_buffer`).
desc(
  `
    Test a very large range of buffer is bound.
    For a render pipeline that use a vertex step mode and a instance step mode vertex buffer, test
    that :
    - For draw, drawIndirect, drawIndexed and drawIndexedIndirect:
        - The bound range of vertex step mode vertex buffer is significantly larger than necessary
        - The bound range of instance step mode vertex buffer is significantly larger than necessary
        - A large buffer is bound to an unused slot
    - For drawIndexed and drawIndexedIndirect:
        - The bound range of index buffer is significantly larger than necessary
    - For drawIndirect and drawIndexedIndirect:
        - The indirect buffer is significantly larger than necessary
`
).
unimplemented();