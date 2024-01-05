/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Buffer Usages Validation Tests in Render Pass and Compute Pass.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { assert, unreachable } from '../../../../../common/util/util.js';
import { ValidationTest } from '../../validation_test.js';

const kBoundBufferSize = 256;










export const kAllBufferUsages = [
'uniform',
'storage',
'read-only-storage',
'vertex',
'index',
'indirect',
'indexedIndirect'];


export class BufferResourceUsageTest extends ValidationTest {
  createBindGroupLayoutForTest(
  type,
  resourceVisibility)
  {
    const bindGroupLayoutEntry = {
      binding: 0,
      visibility:
      resourceVisibility === 'compute' ? GPUShaderStage.COMPUTE : GPUShaderStage.FRAGMENT,
      buffer: {
        type
      }
    };
    return this.device.createBindGroupLayout({
      entries: [bindGroupLayoutEntry]
    });
  }

  createBindGroupForTest(
  buffer,
  offset,
  type,
  resourceVisibility)
  {
    return this.device.createBindGroup({
      layout: this.createBindGroupLayoutForTest(type, resourceVisibility),
      entries: [
      {
        binding: 0,
        resource: { buffer, offset, size: kBoundBufferSize }
      }]

    });
  }

  beginSimpleRenderPass(encoder) {
    const colorTexture = this.device.createTexture({
      format: 'rgba8unorm',
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
      size: [16, 16, 1]
    });
    return encoder.beginRenderPass({
      colorAttachments: [
      {
        view: colorTexture.createView(),
        loadOp: 'load',
        storeOp: 'store'
      }]

    });
  }

  createRenderPipelineForTest(
  pipelineLayout,
  vertexBufferCount)
  {
    const vertexBuffers = [];
    for (let i = 0; i < vertexBufferCount; ++i) {
      vertexBuffers.push({
        arrayStride: 4,
        attributes: [
        {
          format: 'float32',
          shaderLocation: i,
          offset: 0
        }]

      });
    }

    return this.device.createRenderPipeline({
      layout: pipelineLayout,
      vertex: {
        module: this.device.createShaderModule({
          code: this.getNoOpShaderCode('VERTEX')
        }),
        entryPoint: 'main',
        buffers: vertexBuffers
      },
      fragment: {
        module: this.device.createShaderModule({
          code: `
              @fragment fn main()
                -> @location(0) vec4<f32> {
                  return vec4<f32>(0.0, 0.0, 0.0, 1.0);
              }`
        }),
        entryPoint: 'main',
        targets: [{ format: 'rgba8unorm' }]
      },
      primitive: { topology: 'point-list' }
    });
  }
}

function IsBufferUsageInBindGroup(bufferUsage) {
  switch (bufferUsage) {
    case 'uniform':
    case 'storage':
    case 'read-only-storage':
      return true;
    case 'vertex':
    case 'index':
    case 'indirect':
    case 'indexedIndirect':
      return false;
    default:
      unreachable();
  }
}

export const g = makeTestGroup(BufferResourceUsageTest);

g.test('subresources,buffer_usage_in_one_compute_pass_with_no_dispatch').
desc(
  `
Test that it is always allowed to set multiple bind groups with same buffer in a compute pass
encoder without any dispatch calls as state-setting compute pass commands, like setBindGroup(index,
bindGroup, dynamicOffsets), do not contribute directly to a usage scope.`
).
params((u) =>
u.
combine('usage0', ['uniform', 'storage', 'read-only-storage']).
combine('usage1', ['uniform', 'storage', 'read-only-storage']).
beginSubcases().
combine('visibility0', ['compute', 'fragment']).
combine('visibility1', ['compute', 'fragment']).
combine('hasOverlap', [true, false])
).
fn((t) => {
  const { usage0, usage1, visibility0, visibility1, hasOverlap } = t.params;

  const buffer = t.createBufferWithState('valid', {
    size: kBoundBufferSize * 2,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.STORAGE
  });

  const encoder = t.device.createCommandEncoder();
  const computePassEncoder = encoder.beginComputePass();

  const offset0 = 0;
  const bindGroup0 = t.createBindGroupForTest(buffer, offset0, usage0, visibility0);
  computePassEncoder.setBindGroup(0, bindGroup0);

  const offset1 = hasOverlap ? offset0 : kBoundBufferSize;
  const bindGroup1 = t.createBindGroupForTest(buffer, offset1, usage1, visibility1);
  computePassEncoder.setBindGroup(1, bindGroup1);

  computePassEncoder.end();

  t.expectValidationError(() => {
    encoder.finish();
  }, false);
});

g.test('subresources,buffer_usage_in_one_compute_pass_with_one_dispatch').
desc(
  `
Test that when one buffer is used in one compute pass encoder, its list of internal usages within
one usage scope can only be a compatible usage list. According to WebGPU SPEC, within one dispatch,
for each bind group slot that is used by the current GPUComputePipeline's layout, every subresource
referenced by that bind group is "used" in the usage scope.

For both usage === storage, there is writable buffer binding aliasing so we skip this case and will
have tests covered (https://github.com/gpuweb/cts/issues/2232)
`
).
params((u) =>
u.
combine('usage0AccessibleInDispatch', [true, false]).
combine('usage1AccessibleInDispatch', [true, false]).
combine('dispatchBeforeUsage1', [true, false]).
beginSubcases().
combine('usage0', ['uniform', 'storage', 'read-only-storage', 'indirect']).
combine('visibility0', ['compute', 'fragment']).
filter((t) => {
  // The buffer with `indirect` usage is always accessible in the dispatch call.
  if (
  t.usage0 === 'indirect' && (
  !t.usage0AccessibleInDispatch || t.visibility0 !== 'compute' || !t.dispatchBeforeUsage1))
  {
    return false;
  }
  if (t.usage0AccessibleInDispatch && t.visibility0 !== 'compute') {
    return false;
  }
  if (t.dispatchBeforeUsage1 && t.usage1AccessibleInDispatch) {
    return false;
  }
  return true;
}).
combine('usage1', ['uniform', 'storage', 'read-only-storage', 'indirect']).
combine('visibility1', ['compute', 'fragment']).
filter((t) => {
  if (
  t.usage1 === 'indirect' && (
  !t.usage1AccessibleInDispatch || t.visibility1 !== 'compute' || t.dispatchBeforeUsage1))
  {
    return false;
  }
  // When the first buffer usage is `indirect`, there has already been one dispatch call, so
  // in this test we always make the second usage inaccessible in the dispatch call.
  if (
  t.usage1AccessibleInDispatch && (
  t.visibility1 !== 'compute' || t.usage0 === 'indirect'))
  {
    return false;
  }

  // Avoid writable storage buffer bindings aliasing.
  if (t.usage0 === 'storage' && t.usage1 === 'storage') {
    return false;
  }
  return true;
}).
combine('hasOverlap', [true, false])
).
fn((t) => {
  const {
    usage0AccessibleInDispatch,
    usage1AccessibleInDispatch,
    dispatchBeforeUsage1,
    usage0,
    visibility0,
    usage1,
    visibility1,
    hasOverlap
  } = t.params;

  const buffer = t.createBufferWithState('valid', {
    size: kBoundBufferSize * 2,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.STORAGE | GPUBufferUsage.INDIRECT
  });

  const encoder = t.device.createCommandEncoder();
  const computePassEncoder = encoder.beginComputePass();

  const offset0 = 0;
  switch (usage0) {
    case 'uniform':
    case 'storage':
    case 'read-only-storage':{
        const bindGroup0 = t.createBindGroupForTest(buffer, offset0, usage0, visibility0);
        computePassEncoder.setBindGroup(0, bindGroup0);

        /*
         * setBindGroup(bindGroup0);
         * dispatchWorkgroups();
         * setBindGroup(bindGroup1);
         */
        if (dispatchBeforeUsage1) {
          let pipelineLayout = undefined;
          if (usage0AccessibleInDispatch) {
            const bindGroupLayout0 = t.createBindGroupLayoutForTest(usage0, visibility0);
            pipelineLayout = t.device.createPipelineLayout({
              bindGroupLayouts: [bindGroupLayout0]
            });
          }
          const computePipeline = t.createNoOpComputePipeline(pipelineLayout);
          computePassEncoder.setPipeline(computePipeline);
          computePassEncoder.dispatchWorkgroups(1);
        }
        break;
      }
    case 'indirect':{
        /*
         * dispatchWorkgroupsIndirect(buffer);
         * setBindGroup(bindGroup1);
         */
        assert(dispatchBeforeUsage1);
        const computePipeline = t.createNoOpComputePipeline();
        computePassEncoder.setPipeline(computePipeline);
        computePassEncoder.dispatchWorkgroupsIndirect(buffer, offset0);
        break;
      }
  }

  const offset1 = hasOverlap ? offset0 : kBoundBufferSize;
  switch (usage1) {
    case 'uniform':
    case 'storage':
    case 'read-only-storage':{
        const bindGroup1 = t.createBindGroupForTest(buffer, offset1, usage1, visibility1);
        const bindGroupIndex = usage0AccessibleInDispatch ? 1 : 0;
        computePassEncoder.setBindGroup(bindGroupIndex, bindGroup1);

        /*
         * setBindGroup(bindGroup0);
         * setBindGroup(bindGroup1);
         * dispatchWorkgroups();
         */
        if (!dispatchBeforeUsage1) {
          const bindGroupLayouts = [];
          if (usage0AccessibleInDispatch && usage0 !== 'indirect') {
            const bindGroupLayout0 = t.createBindGroupLayoutForTest(usage0, visibility0);
            bindGroupLayouts.push(bindGroupLayout0);
          }
          if (usage1AccessibleInDispatch) {
            const bindGroupLayout1 = t.createBindGroupLayoutForTest(usage1, visibility1);
            bindGroupLayouts.push(bindGroupLayout1);
          }
          const pipelineLayout = bindGroupLayouts ?
          t.device.createPipelineLayout({
            bindGroupLayouts
          }) :
          undefined;
          const computePipeline = t.createNoOpComputePipeline(pipelineLayout);
          computePassEncoder.setPipeline(computePipeline);
          computePassEncoder.dispatchWorkgroups(1);
        }
        break;
      }
    case 'indirect':{
        /*
         * setBindGroup(bindGroup0);
         * dispatchWorkgroupsIndirect(buffer);
         */
        assert(!dispatchBeforeUsage1);
        let pipelineLayout = undefined;
        if (usage0AccessibleInDispatch) {
          assert(usage0 !== 'indirect');
          pipelineLayout = t.device.createPipelineLayout({
            bindGroupLayouts: [t.createBindGroupLayoutForTest(usage0, visibility0)]
          });
        }
        const computePipeline = t.createNoOpComputePipeline(pipelineLayout);
        computePassEncoder.setPipeline(computePipeline);
        computePassEncoder.dispatchWorkgroupsIndirect(buffer, offset1);
        break;
      }
  }
  computePassEncoder.end();

  const usageHasConflict =
  usage0 === 'storage' && usage1 !== 'storage' ||
  usage0 !== 'storage' && usage1 === 'storage';
  const fail =
  usageHasConflict &&
  visibility0 === 'compute' &&
  visibility1 === 'compute' &&
  usage0AccessibleInDispatch &&
  usage1AccessibleInDispatch;
  t.expectValidationError(() => {
    encoder.finish();
  }, fail);
});

g.test('subresources,buffer_usage_in_compute_pass_with_two_dispatches').
desc(
  `
Test that it is always allowed to use one buffer in different dispatch calls as in WebGPU SPEC,
within one dispatch, for each bind group slot that is used by the current GPUComputePipeline's
layout, every subresource referenced by that bind group is "used" in the usage scope, and different
dispatch calls refer to different usage scopes.`
).
params((u) =>
u.
combine('usage0', ['uniform', 'storage', 'read-only-storage', 'indirect']).
combine('usage1', ['uniform', 'storage', 'read-only-storage', 'indirect']).
beginSubcases().
combine('inSamePass', [true, false]).
combine('hasOverlap', [true, false])
).
fn((t) => {
  const { usage0, usage1, inSamePass, hasOverlap } = t.params;

  const UseBufferOnComputePassEncoder = (
  computePassEncoder,
  buffer,
  usage,
  offset) =>
  {
    switch (usage) {
      case 'uniform':
      case 'storage':
      case 'read-only-storage':{
          const bindGroup = t.createBindGroupForTest(buffer, offset, usage, 'compute');
          computePassEncoder.setBindGroup(0, bindGroup);

          const bindGroupLayout = t.createBindGroupLayoutForTest(usage, 'compute');
          const pipelineLayout = t.device.createPipelineLayout({
            bindGroupLayouts: [bindGroupLayout]
          });
          const computePipeline = t.createNoOpComputePipeline(pipelineLayout);
          computePassEncoder.setPipeline(computePipeline);
          computePassEncoder.dispatchWorkgroups(1);
          break;
        }
      case 'indirect':{
          const computePipeline = t.createNoOpComputePipeline();
          computePassEncoder.setPipeline(computePipeline);
          computePassEncoder.dispatchWorkgroupsIndirect(buffer, offset);
          break;
        }
      default:
        unreachable();
        break;
    }
  };

  const buffer = t.createBufferWithState('valid', {
    size: kBoundBufferSize * 2,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.STORAGE | GPUBufferUsage.INDIRECT
  });

  const encoder = t.device.createCommandEncoder();
  const computePassEncoder = encoder.beginComputePass();

  const offset0 = 0;
  const offset1 = hasOverlap ? offset0 : kBoundBufferSize;
  UseBufferOnComputePassEncoder(computePassEncoder, buffer, usage0, offset0);

  if (inSamePass) {
    UseBufferOnComputePassEncoder(computePassEncoder, buffer, usage1, offset1);
    computePassEncoder.end();
  } else {
    computePassEncoder.end();
    const anotherComputePassEncoder = encoder.beginComputePass();
    UseBufferOnComputePassEncoder(anotherComputePassEncoder, buffer, usage1, offset1);
    anotherComputePassEncoder.end();
  }

  t.expectValidationError(() => {
    encoder.finish();
  }, false);
});

g.test('subresources,buffer_usage_in_one_render_pass_with_no_draw').
desc(
  `
Test that when one buffer is used in one render pass encoder, its list of internal usages within one
usage scope (all the commands in the whole render pass) can only be a compatible usage list even if
there is no draw call in the render pass.
    `
).
params((u) =>
u.
combine('usage0', ['uniform', 'storage', 'read-only-storage', 'vertex', 'index']).
combine('usage1', ['uniform', 'storage', 'read-only-storage', 'vertex', 'index']).
beginSubcases().
combine('hasOverlap', [true, false]).
combine('visibility0', ['compute', 'fragment']).
unless((t) => t.visibility0 === 'compute' && !IsBufferUsageInBindGroup(t.usage0)).
combine('visibility1', ['compute', 'fragment']).
unless((t) => t.visibility1 === 'compute' && !IsBufferUsageInBindGroup(t.usage1))
).
fn((t) => {
  const { usage0, usage1, hasOverlap, visibility0, visibility1 } = t.params;

  const UseBufferOnRenderPassEncoder = (
  buffer,
  offset,
  type,
  bindGroupVisibility,
  renderPassEncoder) =>
  {
    switch (type) {
      case 'uniform':
      case 'storage':
      case 'read-only-storage':{
          const bindGroup = t.createBindGroupForTest(buffer, offset, type, bindGroupVisibility);
          renderPassEncoder.setBindGroup(0, bindGroup);
          break;
        }
      case 'vertex':{
          renderPassEncoder.setVertexBuffer(0, buffer, offset, kBoundBufferSize);
          break;
        }
      case 'index':{
          renderPassEncoder.setIndexBuffer(buffer, 'uint16', offset, kBoundBufferSize);
          break;
        }
      case 'indirect':
      case 'indexedIndirect':
        unreachable();
        break;
    }
  };

  const buffer = t.createBufferWithState('valid', {
    size: kBoundBufferSize * 2,
    usage:
    GPUBufferUsage.UNIFORM |
    GPUBufferUsage.STORAGE |
    GPUBufferUsage.VERTEX |
    GPUBufferUsage.INDEX
  });

  const encoder = t.device.createCommandEncoder();
  const renderPassEncoder = t.beginSimpleRenderPass(encoder);
  const offset0 = 0;
  UseBufferOnRenderPassEncoder(buffer, offset0, usage0, visibility0, renderPassEncoder);
  const offset1 = hasOverlap ? offset0 : kBoundBufferSize;
  UseBufferOnRenderPassEncoder(buffer, offset1, usage1, visibility1, renderPassEncoder);
  renderPassEncoder.end();

  const fail = usage0 === 'storage' !== (usage1 === 'storage');
  t.expectValidationError(() => {
    encoder.finish();
  }, fail);
});

g.test('subresources,buffer_usage_in_one_render_pass_with_one_draw').
desc(
  `
Test that when one buffer is used in one render pass encoder where there is one draw call, its list
of internal usages within one usage scope (all the commands in the whole render pass) can only be a
compatible usage list. The usage scope rules are not related to the buffer offset or the bind group
layout visibilities.

For both usage === storage, there is writable buffer binding aliasing so we skip this case and will
have tests covered (https://github.com/gpuweb/cts/issues/2232)
`
).
params((u) =>
u.
combine('usage0', kAllBufferUsages).
combine('usage1', kAllBufferUsages).
beginSubcases().
combine('usage0AccessibleInDraw', [true, false]).
combine('usage1AccessibleInDraw', [true, false]).
combine('drawBeforeUsage1', [true, false]).
combine('visibility0', ['compute', 'fragment']).
filter((t) => {
  // The buffer with `indirect` or `indexedIndirect` usage is always accessible in the draw
  // call.
  if (
  (t.usage0 === 'indirect' || t.usage0 === 'indexedIndirect') && (
  !t.usage0AccessibleInDraw || t.visibility0 !== 'fragment' || !t.drawBeforeUsage1))
  {
    return false;
  }
  // The buffer usages `vertex` and `index` do nothing with shader visibilities.
  if ((t.usage0 === 'vertex' || t.usage0 === 'index') && t.visibility0 !== 'fragment') {
    return false;
  }

  // As usage0 is accessible in the draw call, visibility0 can only be 'fragment'.
  if (t.usage0AccessibleInDraw && t.visibility0 !== 'fragment') {
    return false;
  }
  // As usage1 is accessible in the draw call, the draw call cannot be before usage1.
  if (t.drawBeforeUsage1 && t.usage1AccessibleInDraw) {
    return false;
  }

  // Avoid writable storage buffer bindings aliasing.
  if (t.usage0 === 'storage' && t.usage1 === 'storage') {
    return false;
  }
  return true;
}).
combine('visibility1', ['compute', 'fragment']).
filter((t) => {
  if (
  (t.usage1 === 'indirect' || t.usage1 === 'indexedIndirect') && (
  !t.usage1AccessibleInDraw || t.visibility1 !== 'fragment' || t.drawBeforeUsage1))
  {
    return false;
  }
  if ((t.usage1 === 'vertex' || t.usage1 === 'index') && t.visibility1 !== 'fragment') {
    return false;
  }
  // When the first buffer usage is `indirect` or `indexedIndirect`, there has already been
  // one draw call, so in this test we always make the second usage inaccessible in the draw
  // call.
  if (
  t.usage1AccessibleInDraw && (
  t.visibility1 !== 'fragment' ||
  t.usage0 === 'indirect' ||
  t.usage0 === 'indexedIndirect'))
  {
    return false;
  }
  // When the first buffer usage is `index` and is accessible in the draw call, the second
  // usage cannot be `indirect` (it should be `indexedIndirect` for the tests on indirect draw
  // calls)
  if (t.usage0 === 'index' && t.usage0AccessibleInDraw && t.usage1 === 'indirect') {
    return false;
  }
  return true;
}).
combine('hasOverlap', [true, false])
).
fn((t) => {
  const {
    // Buffer with usage0 will be "used" in the draw call if this value is true.
    usage0AccessibleInDraw,
    // Buffer with usage1 will be "used" in the draw call if this value is true.
    usage1AccessibleInDraw,
    // Whether we will have the draw call before setting the buffer usage as "usage1" or not.
    // If it is true: set-usage0 -> draw -> set-usage1 or indirect-draw -> set-usage1
    // Otherwise: set-usage0 -> set-usage1 -> draw or set-usage0 -> indirect-draw
    drawBeforeUsage1,
    usage0,
    visibility0,
    usage1,
    visibility1,
    hasOverlap
  } = t.params;
  const buffer = t.createBufferWithState('valid', {
    size: kBoundBufferSize * 2,
    usage:
    GPUBufferUsage.UNIFORM |
    GPUBufferUsage.STORAGE |
    GPUBufferUsage.VERTEX |
    GPUBufferUsage.INDEX |
    GPUBufferUsage.INDIRECT
  });

  const UseBufferOnRenderPassEncoder = (
  bufferAccessibleInDraw,
  bufferIndex,
  offset,
  usage,
  bindGroupVisibility,
  renderPassEncoder,
  usedBindGroupLayouts) =>
  {
    switch (usage) {
      case 'uniform':
      case 'storage':
      case 'read-only-storage':{
          const bindGroup = t.createBindGroupForTest(buffer, offset, usage, bindGroupVisibility);
          renderPassEncoder.setBindGroup(bufferIndex, bindGroup);
          // To "use" the bind group we will set the corresponding bind group layout in the
          // pipeline layout when creating the render pipeline.
          if (bufferAccessibleInDraw && bindGroupVisibility === 'fragment') {
            usedBindGroupLayouts.push(t.createBindGroupLayoutForTest(usage, bindGroupVisibility));
          }
          break;
        }
      case 'vertex':{
          renderPassEncoder.setVertexBuffer(bufferIndex, buffer, offset);
          break;
        }
      case 'index':{
          renderPassEncoder.setIndexBuffer(buffer, 'uint16', offset);
          break;
        }
      case 'indirect':
      case 'indexedIndirect':{
          // We will handle the indirect draw calls later.
          break;
        }
    }
  };

  const MakeDrawCallWithOneUsage = (
  usage,
  offset,
  renderPassEncoder) =>
  {
    switch (usage) {
      case 'uniform':
      case 'read-only-storage':
      case 'storage':
      case 'vertex':
        renderPassEncoder.draw(1);
        break;
      case 'index':
        renderPassEncoder.drawIndexed(1);
        break;
      case 'indirect':
        renderPassEncoder.drawIndirect(buffer, offset);
        break;
      case 'indexedIndirect':{
          const indexBuffer = t.device.createBuffer({
            size: 4,
            usage: GPUBufferUsage.INDEX
          });
          renderPassEncoder.setIndexBuffer(indexBuffer, 'uint16');
          renderPassEncoder.drawIndexedIndirect(buffer, offset);
          break;
        }
    }
  };

  const encoder = t.device.createCommandEncoder();
  const renderPassEncoder = t.beginSimpleRenderPass(encoder);

  // Set buffer with usage0
  const offset0 = 0;
  // Invisible bind groups or vertex buffers are all bound to the slot 1.
  const bufferIndex0 = visibility0 === 'fragment' ? 0 : 1;
  const usedBindGroupLayouts = [];

  UseBufferOnRenderPassEncoder(
    usage0AccessibleInDraw,
    bufferIndex0,
    offset0,
    usage0,
    visibility0,
    renderPassEncoder,
    usedBindGroupLayouts
  );

  let vertexBufferCount = 0;

  // Set pipeline and do draw call if drawBeforeUsage1 === true
  if (drawBeforeUsage1) {
    const pipelineLayout = t.device.createPipelineLayout({
      bindGroupLayouts: usedBindGroupLayouts
    });
    // To "use" the vertex buffer we need to set the corresponding vertex buffer layout when
    // creating the render pipeline.
    if (usage0 === 'vertex' && usage0AccessibleInDraw) {
      ++vertexBufferCount;
    }
    const pipeline = t.createRenderPipelineForTest(pipelineLayout, vertexBufferCount);
    renderPassEncoder.setPipeline(pipeline);
    if (!usage0AccessibleInDraw) {
      renderPassEncoder.draw(1);
    } else {
      MakeDrawCallWithOneUsage(usage0, offset0, renderPassEncoder);
    }
  }

  // Set buffer with usage1.
  const offset1 = hasOverlap ? offset0 : kBoundBufferSize;
  let bufferIndex1 = 0;
  if (visibility1 !== 'fragment') {
    // Invisible bind groups or vertex buffers are all bound to the slot 1.
    bufferIndex1 = 1;
  } else if (visibility0 === 'fragment' && usage0AccessibleInDraw) {
    // When buffer is bound to different bind groups or bound as vertex buffers in one render pass
    // encoder, the second buffer binding should consume the slot 1.
    if (IsBufferUsageInBindGroup(usage0) && IsBufferUsageInBindGroup(usage1)) {
      bufferIndex1 = 1;
    } else if (usage0 === 'vertex' && usage1 === 'vertex') {
      bufferIndex1 = 1;
    }
  }

  UseBufferOnRenderPassEncoder(
    usage1AccessibleInDraw,
    bufferIndex1,
    offset1,
    usage1,
    visibility1,
    renderPassEncoder,
    usedBindGroupLayouts
  );

  // Set pipeline and do draw call if drawBeforeUsage1 === false
  if (!drawBeforeUsage1) {
    const pipelineLayout = t.device.createPipelineLayout({
      bindGroupLayouts: usedBindGroupLayouts
    });
    if (usage1 === 'vertex' && usage1AccessibleInDraw) {
      // To "use" the vertex buffer we need to set the corresponding vertex buffer layout when
      // creating the render pipeline.
      ++vertexBufferCount;
    }
    const pipeline = t.createRenderPipelineForTest(pipelineLayout, vertexBufferCount);
    renderPassEncoder.setPipeline(pipeline);

    assert(usage0 !== 'indirect');
    if (!usage0AccessibleInDraw && !usage1AccessibleInDraw) {
      renderPassEncoder.draw(1);
    } else if (usage0AccessibleInDraw && !usage1AccessibleInDraw) {
      MakeDrawCallWithOneUsage(usage0, offset0, renderPassEncoder);
    } else if (!usage0AccessibleInDraw && usage1AccessibleInDraw) {
      MakeDrawCallWithOneUsage(usage1, offset1, renderPassEncoder);
    } else {
      if (usage1 === 'indexedIndirect') {
        // If the index buffer has already been set (as usage0), we won't need to set another
        // index buffer.
        if (usage0 !== 'index') {
          const indexBuffer = t.createBufferWithState('valid', {
            size: 4,
            usage: GPUBufferUsage.INDEX
          });
          renderPassEncoder.setIndexBuffer(indexBuffer, 'uint16');
        }
        renderPassEncoder.drawIndexedIndirect(buffer, offset1);
      } else if (usage1 === 'indirect') {
        assert(usage0 !== 'index');
        renderPassEncoder.drawIndirect(buffer, offset1);
      } else if (usage0 === 'index' || usage1 === 'index') {
        // We need to call drawIndexed to "use" the index buffer (as usage0 or usage1).
        renderPassEncoder.drawIndexed(1);
      } else {
        renderPassEncoder.draw(1);
      }
    }
  }
  renderPassEncoder.end();

  const fail = usage0 === 'storage' !== (usage1 === 'storage');
  t.expectValidationError(() => {
    encoder.finish();
  }, fail);
});

g.test('subresources,buffer_usage_in_one_render_pass_with_two_draws').
desc(
  `
Test that when one buffer is used in different draw calls in one render pass, its list of internal
usages within one usage scope (all the commands in the whole render pass) can only be a compatible
usage list, and the usage scope rules are not related to the buffer offset, while the draw calls in
different render pass encoders belong to different usage scopes.`
).
params((u) =>
u.
combine('usage0', kAllBufferUsages).
combine('usage1', kAllBufferUsages).
beginSubcases().
combine('inSamePass', [true, false]).
combine('hasOverlap', [true, false])
).
fn((t) => {
  const { usage0, usage1, inSamePass, hasOverlap } = t.params;
  const buffer = t.createBufferWithState('valid', {
    size: kBoundBufferSize * 2,
    usage:
    GPUBufferUsage.UNIFORM |
    GPUBufferUsage.STORAGE |
    GPUBufferUsage.VERTEX |
    GPUBufferUsage.INDEX |
    GPUBufferUsage.INDIRECT
  });
  const UseBufferOnRenderPassEncoderInDrawCall = (
  offset,
  usage,
  renderPassEncoder) =>
  {
    switch (usage) {
      case 'uniform':
      case 'storage':
      case 'read-only-storage':{
          const bindGroupLayout = t.createBindGroupLayoutForTest(usage, 'fragment');
          const pipelineLayout = t.device.createPipelineLayout({
            bindGroupLayouts: [bindGroupLayout]
          });
          const pipeline = t.createRenderPipelineForTest(pipelineLayout, 0);
          renderPassEncoder.setPipeline(pipeline);
          const bindGroup = t.createBindGroupForTest(buffer, offset, usage, 'fragment');
          renderPassEncoder.setBindGroup(0, bindGroup);
          renderPassEncoder.draw(1);
          break;
        }
      case 'vertex':{
          const kVertexBufferCount = 1;
          const pipeline = t.createRenderPipelineForTest('auto', kVertexBufferCount);
          renderPassEncoder.setPipeline(pipeline);
          renderPassEncoder.setVertexBuffer(0, buffer, offset);
          renderPassEncoder.draw(1);
          break;
        }
      case 'index':{
          const pipeline = t.createRenderPipelineForTest('auto', 0);
          renderPassEncoder.setPipeline(pipeline);
          renderPassEncoder.setIndexBuffer(buffer, 'uint16', offset);
          renderPassEncoder.drawIndexed(1);
          break;
        }
      case 'indirect':{
          const pipeline = t.createRenderPipelineForTest('auto', 0);
          renderPassEncoder.setPipeline(pipeline);
          renderPassEncoder.drawIndirect(buffer, offset);
          break;
        }
      case 'indexedIndirect':{
          const pipeline = t.createRenderPipelineForTest('auto', 0);
          renderPassEncoder.setPipeline(pipeline);
          const indexBuffer = t.createBufferWithState('valid', {
            size: 4,
            usage: GPUBufferUsage.INDEX
          });
          renderPassEncoder.setIndexBuffer(indexBuffer, 'uint16');
          renderPassEncoder.drawIndexedIndirect(buffer, offset);
          break;
        }
    }
  };

  const encoder = t.device.createCommandEncoder();
  const renderPassEncoder = t.beginSimpleRenderPass(encoder);

  const offset0 = 0;
  UseBufferOnRenderPassEncoderInDrawCall(offset0, usage0, renderPassEncoder);

  const offset1 = hasOverlap ? offset0 : kBoundBufferSize;
  if (inSamePass) {
    UseBufferOnRenderPassEncoderInDrawCall(offset1, usage1, renderPassEncoder);
    renderPassEncoder.end();
  } else {
    renderPassEncoder.end();
    const anotherRenderPassEncoder = t.beginSimpleRenderPass(encoder);
    UseBufferOnRenderPassEncoderInDrawCall(offset1, usage1, anotherRenderPassEncoder);
    anotherRenderPassEncoder.end();
  }

  const fail = inSamePass && usage0 === 'storage' !== (usage1 === 'storage');
  t.expectValidationError(() => {
    encoder.finish();
  }, fail);
});