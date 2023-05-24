/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Here we test the validation for draw functions, mainly the buffer access validation. All four types
of draw calls are tested, and test that validation errors do / don't occur for certain call type
and parameters as expect.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { kVertexFormatInfo } from '../../../../../capability_info.js';

import { ValidationTest } from '../../../validation_test.js';

function callDrawIndexed(test, encoder, drawType, param) {
  switch (drawType) {
    case 'drawIndexed': {
      encoder.drawIndexed(
        param.indexCount,
        param.instanceCount ?? 1,
        param.firstIndex ?? 0,
        param.baseVertex ?? 0,
        param.firstInstance ?? 0
      );

      break;
    }
    case 'drawIndexedIndirect': {
      const indirectArray = new Int32Array([
        param.indexCount,
        param.instanceCount ?? 1,
        param.firstIndex ?? 0,
        param.baseVertex ?? 0,
        param.firstInstance ?? 0,
      ]);

      const indirectBuffer = test.makeBufferWithContents(indirectArray, GPUBufferUsage.INDIRECT);
      encoder.drawIndexedIndirect(indirectBuffer, 0);
      break;
    }
  }
}

function callDraw(test, encoder, drawType, param) {
  switch (drawType) {
    case 'draw': {
      encoder.draw(
        param.vertexCount,
        param.instanceCount ?? 1,
        param.firstVertex ?? 0,
        param.firstInstance ?? 0
      );

      break;
    }
    case 'drawIndirect': {
      const indirectArray = new Int32Array([
        param.vertexCount,
        param.instanceCount ?? 1,
        param.firstVertex ?? 0,
        param.firstInstance ?? 0,
      ]);

      const indirectBuffer = test.makeBufferWithContents(indirectArray, GPUBufferUsage.INDIRECT);
      encoder.drawIndirect(indirectBuffer, 0);
      break;
    }
  }
}

function makeTestPipeline(test, buffers) {
  const bufferLayouts = [];
  for (const b of buffers) {
    bufferLayouts[b.slot] = b;
  }

  return test.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: test.device.createShaderModule({
        code: test.getNoOpShaderCode('VERTEX'),
      }),
      entryPoint: 'main',
      buffers: bufferLayouts,
    },
    fragment: {
      module: test.device.createShaderModule({
        code: test.getNoOpShaderCode('FRAGMENT'),
      }),
      entryPoint: 'main',
      targets: [{ format: 'rgba8unorm', writeMask: 0 }],
    },
    primitive: { topology: 'triangle-list' },
  });
}

function makeTestPipelineWithVertexAndInstanceBuffer(
  test,
  arrayStride,
  attributeFormat,
  attributeOffset = 0
) {
  const vertexBufferLayouts = [
    {
      slot: 1,
      stepMode: 'vertex',
      arrayStride,
      attributes: [
        {
          shaderLocation: 2,
          format: attributeFormat,
          offset: attributeOffset,
        },
      ],
    },
    {
      slot: 7,
      stepMode: 'instance',
      arrayStride,
      attributes: [
        {
          shaderLocation: 6,
          format: attributeFormat,
          offset: attributeOffset,
        },
      ],
    },
  ];

  return makeTestPipeline(test, vertexBufferLayouts);
}

// Default parameters for all kind of draw call, arbitrary non-zero values that is not very large.
const kDefaultParameterForDraw = {
  instanceCount: 100,
  firstInstance: 100,
};

// Default parameters for non-indexed draw, arbitrary non-zero values that is not very large.
const kDefaultParameterForNonIndexedDraw = {
  vertexCount: 100,
  firstVertex: 100,
};

// Default parameters for indexed draw call and required index buffer, arbitrary non-zero values
// that is not very large.
const kDefaultParameterForIndexedDraw = {
  indexCount: 100,
  firstIndex: 100,
  baseVertex: 100,
  indexFormat: 'uint16',
  indexBufferSize: 2 * 200, // exact required bound size for index buffer
};

export const g = makeTestGroup(ValidationTest);

g.test(`unused_buffer_bound`)
  .desc(
    `
In this test we test that a small buffer bound to unused buffer slot won't cause validation error.
- All draw commands,
  - An unused {index , vertex} buffer with uselessly small range is bound (immediately before draw
    call)
`
  )
  .params(u =>
    u //
      .combine('smallIndexBuffer', [false, true])
      .combine('smallVertexBuffer', [false, true])
      .combine('smallInstanceBuffer', [false, true])
      .beginSubcases()
      .combine('drawType', ['draw', 'drawIndexed', 'drawIndirect', 'drawIndexedIndirect'])
      .unless(
        // Always provide index buffer of enough size if it is used by indexed draw
        p =>
          p.smallIndexBuffer &&
          (p.drawType === 'drawIndexed' || p.drawType === 'drawIndexedIndirect')
      )
      .combine('bufferOffset', [0, 4])
      .combine('boundSize', [0, 1])
  )
  .fn(t => {
    const {
      smallIndexBuffer,
      smallVertexBuffer,
      smallInstanceBuffer,
      drawType,
      bufferOffset,
      boundSize,
    } = t.params;
    const renderPipeline = t.createNoOpRenderPipeline();
    const bufferSize = bufferOffset + boundSize;
    const smallBuffer = t.createBufferWithState('valid', {
      size: bufferSize,
      usage: GPUBufferUsage.INDEX | GPUBufferUsage.VERTEX,
    });

    // An index buffer of enough size, used if smallIndexBuffer === false
    const { indexFormat, indexBufferSize } = kDefaultParameterForIndexedDraw;
    const indexBuffer = t.createBufferWithState('valid', {
      size: indexBufferSize,
      usage: GPUBufferUsage.INDEX,
    });

    for (const encoderType of ['render bundle', 'render pass']) {
      for (const setPipelineBeforeBuffer of [false, true]) {
        const commandBufferMaker = t.createEncoder(encoderType);
        const renderEncoder = commandBufferMaker.encoder;

        if (setPipelineBeforeBuffer) {
          renderEncoder.setPipeline(renderPipeline);
        }

        if (drawType === 'drawIndexed' || drawType === 'drawIndexedIndirect') {
          // Always use large enough index buffer for indexed draw. Index buffer OOB validation is
          // tested in index_buffer_OOB.
          renderEncoder.setIndexBuffer(indexBuffer, indexFormat, 0, indexBufferSize);
        } else if (smallIndexBuffer) {
          renderEncoder.setIndexBuffer(smallBuffer, indexFormat, bufferOffset, boundSize);
        }
        if (smallVertexBuffer) {
          renderEncoder.setVertexBuffer(1, smallBuffer, bufferOffset, boundSize);
        }
        if (smallInstanceBuffer) {
          renderEncoder.setVertexBuffer(7, smallBuffer, bufferOffset, boundSize);
        }

        if (!setPipelineBeforeBuffer) {
          renderEncoder.setPipeline(renderPipeline);
        }

        if (drawType === 'draw' || drawType === 'drawIndirect') {
          const drawParam = {
            ...kDefaultParameterForDraw,
            ...kDefaultParameterForNonIndexedDraw,
          };
          callDraw(t, renderEncoder, drawType, drawParam);
        } else {
          const drawParam = {
            ...kDefaultParameterForDraw,
            ...kDefaultParameterForIndexedDraw,
          };
          callDrawIndexed(t, renderEncoder, drawType, drawParam);
        }

        // Binding a unused small index/vertex buffer will never cause validation error.
        commandBufferMaker.validateFinishAndSubmit(true, true);
      }
    }
  });

g.test(`index_buffer_OOB`)
  .desc(
    `
In this test we test that index buffer OOB is caught as a validation error in drawIndexed, but not in
drawIndexedIndirect as it is GPU-validated.
- Issue an indexed draw call, with the following index buffer states, for {all index formats}:
    - range and GPUBuffer are exactly the required size for the draw call
    - range is too small but GPUBuffer is still large enough
    - range and GPUBuffer are both too small
`
  )
  .params(u =>
    u
      .combine('bufferSizeInElements', [10, 100])
      // Binding size is always no larger than buffer size, make sure that setIndexBuffer succeed
      .combine('bindingSizeInElements', [10])
      .combine('drawIndexCount', [10, 11])
      .combine('drawType', ['drawIndexed', 'drawIndexedIndirect'])
      .beginSubcases()
      .combine('indexFormat', ['uint16', 'uint32'])
  )
  .fn(t => {
    const {
      indexFormat,
      bindingSizeInElements,
      bufferSizeInElements,
      drawIndexCount,
      drawType,
    } = t.params;

    const indexElementSize = indexFormat === 'uint16' ? 2 : 4;
    const bindingSize = bindingSizeInElements * indexElementSize;
    const bufferSize = bufferSizeInElements * indexElementSize;

    const desc = {
      size: bufferSize,
      usage: GPUBufferUsage.INDEX | GPUBufferUsage.COPY_DST,
    };
    const indexBuffer = t.createBufferWithState('valid', desc);

    const drawCallParam = {
      indexCount: drawIndexCount,
    };

    // Encoder finish will succeed if no index buffer access OOB when calling drawIndexed,
    // and always succeed when calling drawIndexedIndirect.
    const isFinishSuccess =
      drawIndexCount <= bindingSizeInElements || drawType === 'drawIndexedIndirect';

    const renderPipeline = t.createNoOpRenderPipeline();

    for (const encoderType of ['render bundle', 'render pass']) {
      for (const setPipelineBeforeBuffer of [false, true]) {
        const commandBufferMaker = t.createEncoder(encoderType);
        const renderEncoder = commandBufferMaker.encoder;

        if (setPipelineBeforeBuffer) {
          renderEncoder.setPipeline(renderPipeline);
        }
        renderEncoder.setIndexBuffer(indexBuffer, indexFormat, 0, bindingSize);
        if (!setPipelineBeforeBuffer) {
          renderEncoder.setPipeline(renderPipeline);
        }

        callDrawIndexed(t, renderEncoder, drawType, drawCallParam);

        commandBufferMaker.validateFinishAndSubmit(isFinishSuccess, true);
      }
    }
  });

g.test(`vertex_buffer_OOB`)
  .desc(
    `
In this test we test the vertex buffer OOB validation in draw calls. Specifically, only vertex step
mode buffer OOB in draw and instance step mode buffer OOB in draw and drawIndexed are CPU-validated.
Other cases are handled by robust access and no validation error occurs.
- Test that:
    - Draw call needs to read {=, >} any bound vertex buffer range, with GPUBuffer that is {large
      enough, exactly the size of bound range}
        - Binding size = 0 (ensure it's not treated as a special case)
        - x= weird buffer offset values
        - x= weird attribute offset values
        - x= weird arrayStride values
        - x= {render pass, render bundle}
- For vertex step mode vertex buffer,
    - Test that:
        - vertexCount largeish
        - firstVertex {=, >} 0
        - arrayStride is 0 and bound buffer size too small
        - (vertexCount + firstVertex) is zero
    - Validation error occurs in:
        - draw
        - drawIndexed with a zero array stride vertex step mode buffer OOB
    - Otherwise no validation error in drawIndexed, draIndirect and drawIndexedIndirect
- For instance step mode vertex buffer,
    - Test with draw and drawIndexed:
        - instanceCount largeish
        - firstInstance {=, >} 0
        - arrayStride is 0 and bound buffer size too small
        - (instanceCount + firstInstance) is zero
    - Validation error occurs in draw and drawIndexed
    - No validation error in drawIndirect and drawIndexedIndirect

In this test, we use a a render pipeline requiring one vertex step mode with different vertex buffer
layouts (attribute offset, array stride, vertex format). Then for a given drawing parameter set (e.g.,
vertexCount, instanceCount, firstVertex, indexCount), we calculate the exactly required size for
vertex step mode vertex buffer. Then, we generate buffer parameters (i.e. GPU buffer size,
binding offset and binding size) for all buffers, covering both (bound size == required size),
(bound size == required size - 1), and (bound size == 0), and test that draw and drawIndexed will
success/error as expected. Such set of buffer parameters should include cases like weird offset values.
`
  )
  .params(u =>
    u
      // type of draw call
      .combine('type', ['draw', 'drawIndexed', 'drawIndirect', 'drawIndexedIndirect'])
      // the state of vertex step mode vertex buffer bound size
      .combine('VBSize', ['zero', 'exile', 'enough'])
      // the state of instance step mode vertex buffer bound size
      .combine('IBSize', ['zero', 'exile', 'enough'])
      // should the vertex stride count be zero
      .combine('VStride0', [false, true])
      // should the instance stride count be zero
      .combine('IStride0', [false, true])
      // the state of array stride
      .combine('AStride', ['zero', 'exact', 'oversize'])
      // the factor for offset of attributes in vertex layout
      .combine('offset', [0, 1, 2, 7]) // the offset of attribute will be factor * MIN(4, sizeof(vertexFormat))
      .beginSubcases()
      .combine('setBufferOffset', [0, 200]) // must be a multiple of 4
      .combine('attributeFormat', ['snorm8x2', 'float32', 'float16x4'])
      .combine('vertexCount', [0, 1, 10000])
      .combine('firstVertex', [0, 10000])
      .filter(p => p.VStride0 === (p.firstVertex + p.vertexCount === 0))
      .combine('instanceCount', [0, 1, 10000])
      .combine('firstInstance', [0, 10000])
      .filter(p => p.IStride0 === (p.firstInstance + p.instanceCount === 0))
      .unless(p => p.vertexCount === 10000 && p.instanceCount === 10000)
  )
  .fn(t => {
    const {
      type: drawType,
      VBSize: boundVertexBufferSizeState,
      IBSize: boundInstanceBufferSizeState,
      VStride0: zeroVertexStrideCount,
      IStride0: zeroInstanceStrideCount,
      AStride: arrayStrideState,
      offset: attributeOffsetFactor,
      setBufferOffset,
      attributeFormat,
      vertexCount,
      instanceCount,
      firstVertex,
      firstInstance,
    } = t.params;

    const attributeFormatInfo = kVertexFormatInfo[attributeFormat];
    const formatSize = attributeFormatInfo.bytesPerComponent * attributeFormatInfo.componentCount;
    const attributeOffset = attributeOffsetFactor * Math.min(4, formatSize);
    const lastStride = attributeOffset + formatSize;
    let arrayStride = 0;
    if (arrayStrideState !== 'zero') {
      arrayStride = lastStride;
      if (arrayStrideState === 'oversize') {
        // Add an arbitrary number to array stride to make it larger than required by attributes
        arrayStride = arrayStride + 20;
      }
      arrayStride = arrayStride + (-arrayStride & 3); // Make sure arrayStride is a multiple of 4
    }

    const calcSetBufferSize = (boundBufferSizeState, strideCount) => {
      let requiredBufferSize;
      if (strideCount > 0) {
        requiredBufferSize = arrayStride * (strideCount - 1) + lastStride;
      } else {
        // Spec do not validate bounded buffer size if strideCount == 0.
        requiredBufferSize = lastStride;
      }
      let setBufferSize;
      switch (boundBufferSizeState) {
        case 'zero': {
          setBufferSize = 0;
          break;
        }
        case 'exile': {
          setBufferSize = requiredBufferSize - 1;
          break;
        }
        case 'enough': {
          setBufferSize = requiredBufferSize;
          break;
        }
      }

      return setBufferSize;
    };

    const strideCountForVertexBuffer = firstVertex + vertexCount;
    const setVertexBufferSize = calcSetBufferSize(
      boundVertexBufferSizeState,
      strideCountForVertexBuffer
    );

    const vertexBufferSize = setBufferOffset + setVertexBufferSize;
    const strideCountForInstanceBuffer = firstInstance + instanceCount;
    const setInstanceBufferSize = calcSetBufferSize(
      boundInstanceBufferSizeState,
      strideCountForInstanceBuffer
    );

    const instanceBufferSize = setBufferOffset + setInstanceBufferSize;

    const vertexBuffer = t.createBufferWithState('valid', {
      size: vertexBufferSize,
      usage: GPUBufferUsage.VERTEX,
    });
    const instanceBuffer = t.createBufferWithState('valid', {
      size: instanceBufferSize,
      usage: GPUBufferUsage.VERTEX,
    });

    const renderPipeline = makeTestPipelineWithVertexAndInstanceBuffer(
      t,
      arrayStride,
      attributeFormat,
      attributeOffset
    );

    for (const encoderType of ['render bundle', 'render pass']) {
      for (const setPipelineBeforeBuffer of [false, true]) {
        const commandBufferMaker = t.createEncoder(encoderType);
        const renderEncoder = commandBufferMaker.encoder;

        if (setPipelineBeforeBuffer) {
          renderEncoder.setPipeline(renderPipeline);
        }
        renderEncoder.setVertexBuffer(1, vertexBuffer, setBufferOffset, setVertexBufferSize);
        renderEncoder.setVertexBuffer(7, instanceBuffer, setBufferOffset, setInstanceBufferSize);
        if (!setPipelineBeforeBuffer) {
          renderEncoder.setPipeline(renderPipeline);
        }

        if (drawType === 'draw' || drawType === 'drawIndirect') {
          const drawParam = {
            vertexCount,
            instanceCount,
            firstVertex,
            firstInstance,
          };

          callDraw(t, renderEncoder, drawType, drawParam);
        } else {
          const {
            indexFormat,
            indexCount,
            firstIndex,
            indexBufferSize,
          } = kDefaultParameterForIndexedDraw;

          const desc = {
            size: indexBufferSize,
            usage: GPUBufferUsage.INDEX | GPUBufferUsage.COPY_DST,
          };
          const indexBuffer = t.createBufferWithState('valid', desc);

          const drawParam = {
            indexCount,
            instanceCount,
            firstIndex,
            baseVertex: firstVertex,
            firstInstance,
          };

          renderEncoder.setIndexBuffer(indexBuffer, indexFormat, 0, indexBufferSize);
          callDrawIndexed(t, renderEncoder, drawType, drawParam);
        }

        const isVertexBufferOOB =
          boundVertexBufferSizeState !== 'enough' &&
          drawType === 'draw' && // drawIndirect, drawIndexed, and drawIndexedIndirect do not validate vertex step mode buffer
          !zeroVertexStrideCount; // vertex step mode buffer never OOB if stride count = 0
        const isInstanceBufferOOB =
          boundInstanceBufferSizeState !== 'enough' &&
          (drawType === 'draw' || drawType === 'drawIndexed') && // drawIndirect and drawIndexedIndirect do not validate instance step mode buffer
          !zeroInstanceStrideCount; // vertex step mode buffer never OOB if stride count = 0
        const isFinishSuccess = !isVertexBufferOOB && !isInstanceBufferOOB;

        commandBufferMaker.validateFinishAndSubmit(isFinishSuccess, true);
      }
    }
  });

g.test(`buffer_binding_overlap`)
  .desc(
    `
In this test we test that binding one GPU buffer to multiple vertex buffer slot or both vertex
buffer slot and index buffer will cause no validation error, with completely/partial overlap.
    - x= all draw types
`
  )
  .params(u =>
    u //
      .combine('drawType', ['draw', 'drawIndexed', 'drawIndirect', 'drawIndexedIndirect'])
      .beginSubcases()
      .combine('vertexBoundOffestFactor', [0, 0.5, 1, 1.5, 2])
      .combine('instanceBoundOffestFactor', [0, 0.5, 1, 1.5, 2])
      .combine('indexBoundOffestFactor', [0, 0.5, 1, 1.5, 2])
      .combine('arrayStrideState', ['zero', 'exact', 'oversize'])
  )
  .fn(t => {
    const {
      drawType,
      vertexBoundOffestFactor,
      instanceBoundOffestFactor,
      indexBoundOffestFactor,
      arrayStrideState,
    } = t.params;

    // Compute the array stride for vertex step mode and instance step mode attribute
    const attributeFormat = 'float32x4';
    const attributeFormatInfo = kVertexFormatInfo[attributeFormat];
    const formatSize = attributeFormatInfo.bytesPerComponent * attributeFormatInfo.componentCount;
    const attributeOffset = 0;
    const lastStride = attributeOffset + formatSize;
    let arrayStride = 0;
    if (arrayStrideState !== 'zero') {
      arrayStride = lastStride;
      if (arrayStrideState === 'oversize') {
        // Add an arbitrary number to array stride
        arrayStride = arrayStride + 20;
      }
      arrayStride = arrayStride + (-arrayStride & 3); // Make sure arrayStride is a multiple of 4
    }

    const calcAttributeBufferSize = strideCount => {
      let requiredBufferSize;
      if (strideCount > 0) {
        requiredBufferSize = arrayStride * (strideCount - 1) + lastStride;
      } else {
        // Spec do not validate bounded buffer size if strideCount == 0.
        requiredBufferSize = lastStride;
      }
      return requiredBufferSize;
    };

    const calcSetBufferOffset = (requiredSetBufferSize, offsetFactor) => {
      const offset = Math.ceil(requiredSetBufferSize * offsetFactor);
      const alignedOffset = offset + (-offset & 3); // Make sure offset is a multiple of 4
      return alignedOffset;
    };

    // Compute required bound range for all vertex and index buffer to ensure the shared GPU buffer
    // has enough size.
    const { vertexCount, firstVertex } = kDefaultParameterForNonIndexedDraw;
    const strideCountForVertexBuffer = firstVertex + vertexCount;
    const setVertexBufferSize = calcAttributeBufferSize(strideCountForVertexBuffer);
    const setVertexBufferOffset = calcSetBufferOffset(setVertexBufferSize, vertexBoundOffestFactor);
    let requiredBufferSize = setVertexBufferOffset + setVertexBufferSize;

    const { instanceCount, firstInstance } = kDefaultParameterForDraw;
    const strideCountForInstanceBuffer = firstInstance + instanceCount;
    const setInstanceBufferSize = calcAttributeBufferSize(strideCountForInstanceBuffer);
    const setInstanceBufferOffset = calcSetBufferOffset(
      setInstanceBufferSize,
      instanceBoundOffestFactor
    );

    requiredBufferSize = Math.max(
      requiredBufferSize,
      setInstanceBufferOffset + setInstanceBufferSize
    );

    const { indexBufferSize: setIndexBufferSize, indexFormat } = kDefaultParameterForIndexedDraw;
    const setIndexBufferOffset = calcSetBufferOffset(setIndexBufferSize, indexBoundOffestFactor);
    requiredBufferSize = Math.max(requiredBufferSize, setIndexBufferOffset + setIndexBufferSize);

    // Create the shared GPU buffer with both vertetx and index usage
    const sharedBuffer = t.createBufferWithState('valid', {
      size: requiredBufferSize,
      usage: GPUBufferUsage.VERTEX | GPUBufferUsage.INDEX,
    });

    const renderPipeline = makeTestPipelineWithVertexAndInstanceBuffer(
      t,
      arrayStride,
      attributeFormat
    );

    for (const encoderType of ['render bundle', 'render pass']) {
      for (const setPipelineBeforeBuffer of [false, true]) {
        const commandBufferMaker = t.createEncoder(encoderType);
        const renderEncoder = commandBufferMaker.encoder;

        if (setPipelineBeforeBuffer) {
          renderEncoder.setPipeline(renderPipeline);
        }
        renderEncoder.setVertexBuffer(1, sharedBuffer, setVertexBufferOffset, setVertexBufferSize);
        renderEncoder.setVertexBuffer(
          7,
          sharedBuffer,
          setInstanceBufferOffset,
          setInstanceBufferSize
        );

        renderEncoder.setIndexBuffer(
          sharedBuffer,
          indexFormat,
          setIndexBufferOffset,
          setIndexBufferSize
        );

        if (!setPipelineBeforeBuffer) {
          renderEncoder.setPipeline(renderPipeline);
        }

        if (drawType === 'draw' || drawType === 'drawIndirect') {
          const drawParam = {
            ...kDefaultParameterForDraw,
            ...kDefaultParameterForNonIndexedDraw,
          };
          callDraw(t, renderEncoder, drawType, drawParam);
        } else {
          const drawParam = {
            ...kDefaultParameterForDraw,
            ...kDefaultParameterForIndexedDraw,
          };
          callDrawIndexed(t, renderEncoder, drawType, drawParam);
        }

        // Since all bound buffer are of enough size, draw call should always succeed.
        commandBufferMaker.validateFinishAndSubmit(true, true);
      }
    }
  });

g.test(`last_buffer_setting_take_account`)
  .desc(
    `
In this test we test that only the last setting for a buffer slot take account.
- All (non/indexed, in/direct) draw commands
  - setPl, setVB, setIB, draw, {setPl,setVB,setIB,nothing (control)}, then a larger draw that
    wouldn't have been valid before that
`
  )
  .unimplemented();

g.test(`max_draw_count`)
  .desc(
    `
In this test we test that draw count which exceeds
GPURenderPassDescriptor.maxDrawCount causes validation error on
GPUCommandEncoder.finish(). The test sets specified maxDrawCount,
calls specified draw call specified times with or without bundles,
and checks whether GPUCommandEncoder.finish() causes a validation error.
    - x= whether to use a bundle for the first half of the draw calls
    - x= whether to use a bundle for the second half of the draw calls
    - x= several different draw counts
    - x= several different maxDrawCounts
`
  )
  .params(u =>
    u
      .combine('bundleFirstHalf', [false, true])
      .combine('bundleSecondHalf', [false, true])
      .combine('maxDrawCount', [0, 1, 4, 16])
      .beginSubcases()
      .expand('drawCount', p => new Set([0, p.maxDrawCount, p.maxDrawCount + 1]))
  )
  .fn(t => {
    const { bundleFirstHalf, bundleSecondHalf, maxDrawCount, drawCount } = t.params;

    const colorFormat = 'rgba8unorm';
    const colorTexture = t.device.createTexture({
      size: { width: 1, height: 1, depthOrArrayLayers: 1 },
      format: colorFormat,
      mipLevelCount: 1,
      sampleCount: 1,
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });

    const pipeline = t.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: t.device.createShaderModule({
          code: `
            @vertex fn main() -> @builtin(position) vec4<f32> {
              return vec4<f32>();
            }
          `,
        }),
        entryPoint: 'main',
      },
      fragment: {
        module: t.device.createShaderModule({
          code: `@fragment fn main() {}`,
        }),
        entryPoint: 'main',
        targets: [{ format: colorFormat, writeMask: 0 }],
      },
    });

    const indexBuffer = t.makeBufferWithContents(new Uint16Array([0, 0, 0]), GPUBufferUsage.INDEX);
    const indirectBuffer = t.makeBufferWithContents(
      new Uint32Array([3, 1, 0, 0]),
      GPUBufferUsage.INDIRECT
    );

    const indexedIndirectBuffer = t.makeBufferWithContents(
      new Uint32Array([3, 1, 0, 0, 0]),
      GPUBufferUsage.INDIRECT
    );

    const commandEncoder = t.device.createCommandEncoder();
    const renderPassEncoder = commandEncoder.beginRenderPass({
      colorAttachments: [
        {
          view: colorTexture.createView(),
          loadOp: 'clear',
          storeOp: 'store',
        },
      ],

      maxDrawCount,
    });

    const firstHalfEncoder = bundleFirstHalf
      ? t.device.createRenderBundleEncoder({
          colorFormats: [colorFormat],
        })
      : renderPassEncoder;

    const secondHalfEncoder = bundleSecondHalf
      ? t.device.createRenderBundleEncoder({
          colorFormats: [colorFormat],
        })
      : renderPassEncoder;

    firstHalfEncoder.setPipeline(pipeline);
    firstHalfEncoder.setIndexBuffer(indexBuffer, 'uint16');
    secondHalfEncoder.setPipeline(pipeline);
    secondHalfEncoder.setIndexBuffer(indexBuffer, 'uint16');

    const halfDrawCount = Math.floor(drawCount / 2);
    for (let i = 0; i < drawCount; i++) {
      const encoder = i < halfDrawCount ? firstHalfEncoder : secondHalfEncoder;
      if (i % 4 === 0) {
        encoder.draw(3);
      }
      if (i % 4 === 1) {
        encoder.drawIndexed(3);
      }
      if (i % 4 === 2) {
        encoder.drawIndirect(indirectBuffer, 0);
      }
      if (i % 4 === 3) {
        encoder.drawIndexedIndirect(indexedIndirectBuffer, 0);
      }
    }

    const bundles = [];
    if (bundleFirstHalf) {
      bundles.push(firstHalfEncoder.finish());
    }
    if (bundleSecondHalf) {
      bundles.push(secondHalfEncoder.finish());
    }

    if (bundles.length > 0) {
      renderPassEncoder.executeBundles(bundles);
    }

    renderPassEncoder.end();

    t.expectValidationError(() => {
      commandEncoder.finish();
    }, drawCount > maxDrawCount);
  });
