/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test other buffer usage validation rules that are not tests in ./in_pass_encoder.spec.js.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { unreachable } from '../../../../../common/util/util.js';

import { BufferResourceUsageTest, kAllBufferUsages } from './in_pass_encoder.spec.js';

export const g = makeTestGroup(BufferResourceUsageTest);

const kBufferSize = 256;

g.test('subresources,reset_buffer_usage_before_dispatch').
desc(
  `
Test that the buffer usages which are reset by another state-setting commands before a dispatch call
do not contribute directly to any usage scope in a compute pass.`
).
params((u) =>
u.
combine('usage0', ['uniform', 'storage', 'read-only-storage']).
combine('usage1', ['uniform', 'storage', 'read-only-storage', 'indirect'])
).
fn((t) => {
  const { usage0, usage1 } = t.params;

  const kUsages = GPUBufferUsage.UNIFORM | GPUBufferUsage.STORAGE | GPUBufferUsage.INDIRECT;
  const buffer = t.createBufferWithState('valid', {
    size: kBufferSize,
    usage: kUsages
  });
  const anotherBuffer = t.createBufferWithState('valid', {
    size: kBufferSize,
    usage: kUsages
  });

  const bindGroupLayouts = [
  t.createBindGroupLayoutForTest(usage0, 'compute')];

  if (usage1 !== 'indirect') {
    bindGroupLayouts.push(t.createBindGroupLayoutForTest(usage1, 'compute'));
  }
  const pipelineLayout = t.device.createPipelineLayout({ bindGroupLayouts });
  const computePipeline = t.createNoOpComputePipeline(pipelineLayout);

  const encoder = t.device.createCommandEncoder();
  const computePassEncoder = encoder.beginComputePass();
  computePassEncoder.setPipeline(computePipeline);

  // Set usage0 for buffer at bind group index 0
  const bindGroup0 = t.createBindGroupForTest(buffer, 0, usage0, 'compute');
  computePassEncoder.setBindGroup(0, bindGroup0);

  // Reset bind group index 0 with another bind group that uses anotherBuffer
  const anotherBindGroup = t.createBindGroupForTest(anotherBuffer, 0, usage0, 'compute');
  computePassEncoder.setBindGroup(0, anotherBindGroup);

  // Set usage1 for buffer
  switch (usage1) {
    case 'uniform':
    case 'storage':
    case 'read-only-storage':{
        const bindGroup1 = t.createBindGroupForTest(buffer, 0, usage1, 'compute');
        computePassEncoder.setBindGroup(1, bindGroup1);
        computePassEncoder.dispatchWorkgroups(1);
        break;
      }
    case 'indirect':{
        computePassEncoder.dispatchWorkgroupsIndirect(buffer, 0);
        break;
      }
  }
  computePassEncoder.end();

  t.expectValidationError(() => {
    encoder.finish();
  }, false);
});

g.test('subresources,reset_buffer_usage_before_draw').
desc(
  `
Test that the buffer usages which are reset by another state-setting commands before a draw call
still contribute directly to the usage scope of the draw call.`
).
params((u) =>
u.
combine('usage0', ['uniform', 'storage', 'read-only-storage', 'vertex', 'index']).
combine('usage1', kAllBufferUsages).
unless((t) => {
  return t.usage0 === 'index' && t.usage1 === 'indirect';
})
).
fn((t) => {
  const { usage0, usage1 } = t.params;

  const kUsages =
  GPUBufferUsage.UNIFORM |
  GPUBufferUsage.STORAGE |
  GPUBufferUsage.INDIRECT |
  GPUBufferUsage.VERTEX |
  GPUBufferUsage.INDEX;
  const buffer = t.createBufferWithState('valid', {
    size: kBufferSize,
    usage: kUsages
  });
  const anotherBuffer = t.createBufferWithState('valid', {
    size: kBufferSize,
    usage: kUsages
  });

  const encoder = t.device.createCommandEncoder();
  const renderPassEncoder = t.beginSimpleRenderPass(encoder);

  const bindGroupLayouts = [];
  let vertexBufferCount = 0;

  // Set buffer as usage0 and reset buffer with anotherBuffer as usage0
  switch (usage0) {
    case 'uniform':
    case 'storage':
    case 'read-only-storage':{
        const bindGroup0 = t.createBindGroupForTest(buffer, 0, usage0, 'fragment');
        renderPassEncoder.setBindGroup(bindGroupLayouts.length, bindGroup0);

        const anotherBindGroup = t.createBindGroupForTest(anotherBuffer, 0, usage0, 'fragment');
        renderPassEncoder.setBindGroup(bindGroupLayouts.length, anotherBindGroup);

        bindGroupLayouts.push(t.createBindGroupLayoutForTest(usage0, 'fragment'));
        break;
      }
    case 'vertex':{
        renderPassEncoder.setVertexBuffer(vertexBufferCount, buffer);
        renderPassEncoder.setVertexBuffer(vertexBufferCount, anotherBuffer);

        ++vertexBufferCount;
        break;
      }
    case 'index':{
        renderPassEncoder.setIndexBuffer(buffer, 'uint16');
        renderPassEncoder.setIndexBuffer(anotherBuffer, 'uint16');
        break;
      }
  }

  // Set buffer as usage1
  switch (usage1) {
    case 'uniform':
    case 'storage':
    case 'read-only-storage':{
        const bindGroup1 = t.createBindGroupForTest(buffer, 0, usage1, 'fragment');
        renderPassEncoder.setBindGroup(bindGroupLayouts.length, bindGroup1);

        bindGroupLayouts.push(t.createBindGroupLayoutForTest(usage1, 'fragment'));
        break;
      }
    case 'vertex':{
        renderPassEncoder.setVertexBuffer(vertexBufferCount, buffer);
        ++vertexBufferCount;
        break;
      }
    case 'index':{
        renderPassEncoder.setIndexBuffer(buffer, 'uint16');
        break;
      }
    case 'indirect':
    case 'indexedIndirect':
      break;
  }

  // Add draw call
  const pipelineLayout = t.device.createPipelineLayout({
    bindGroupLayouts
  });
  const renderPipeline = t.createRenderPipelineForTest(pipelineLayout, vertexBufferCount);
  renderPassEncoder.setPipeline(renderPipeline);
  switch (usage1) {
    case 'indexedIndirect':{
        if (usage0 !== 'index') {
          const indexBuffer = t.createBufferWithState('valid', {
            size: 4,
            usage: GPUBufferUsage.INDEX
          });
          renderPassEncoder.setIndexBuffer(indexBuffer, 'uint16');
        }
        renderPassEncoder.drawIndexedIndirect(buffer, 0);
        break;
      }
    case 'indirect':{
        renderPassEncoder.drawIndirect(buffer, 0);
        break;
      }
    case 'index':{
        renderPassEncoder.drawIndexed(1);
        break;
      }
    case 'vertex':
    case 'uniform':
    case 'storage':
    case 'read-only-storage':{
        if (usage0 === 'index') {
          renderPassEncoder.drawIndexed(1);
        } else {
          renderPassEncoder.draw(1);
        }
        break;
      }
  }

  renderPassEncoder.end();

  const fail = usage0 === 'storage' !== (usage1 === 'storage');
  t.expectValidationError(() => {
    encoder.finish();
  }, fail);
});

g.test('subresources,buffer_usages_in_copy_and_pass').
desc(
  `
  Test that using one buffer in a copy command, a render or compute pass encoder is always allowed
  as WebGPU SPEC (chapter 3.4.5) defines that out of any pass encoder, each command belongs to one
  separated usage scope.`
).
params((u) =>
u.
combine('usage0', [
'copy-src',
'copy-dst',
'uniform',
'storage',
'read-only-storage',
'vertex',
'index',
'indirect',
'indexedIndirect']
).
combine('usage1', [
'copy-src',
'copy-dst',
'uniform',
'storage',
'read-only-storage',
'vertex',
'index',
'indirect',
'indexedIndirect']
).
combine('pass', ['render', 'compute']).
unless(({ usage0, usage1, pass }) => {
  const IsCopy = (usage) => {
    return usage === 'copy-src' || usage === 'copy-dst';
  };
  // We intend to test copy usages in this test.
  if (!IsCopy(usage0) && !IsCopy(usage1)) {
    return true;
  }
  // When both usage0 and usage1 are copy usages, 'pass' is meaningless so in such situation
  // we just need to reserve one value as 'pass'.
  if (IsCopy(usage0) && IsCopy(usage1)) {
    return pass === 'compute';
  }

  const IsValidComputeUsage = (usage) => {
    switch (usage) {
      case 'vertex':
      case 'index':
      case 'indexedIndirect':
        return false;
      default:
        return true;
    }
  };
  if (pass === 'compute') {
    return !IsValidComputeUsage(usage0) || !IsValidComputeUsage(usage1);
  }

  return false;
})
).
fn((t) => {
  const { usage0, usage1, pass } = t.params;

  const kUsages =
  GPUBufferUsage.COPY_SRC |
  GPUBufferUsage.COPY_DST |
  GPUBufferUsage.UNIFORM |
  GPUBufferUsage.STORAGE |
  GPUBufferUsage.INDIRECT |
  GPUBufferUsage.VERTEX |
  GPUBufferUsage.INDEX;
  const buffer = t.createBufferWithState('valid', {
    size: kBufferSize,
    usage: kUsages
  });

  const UseBufferOnCommandEncoder = (
  usage,









  encoder) =>
  {
    switch (usage) {
      case 'copy-src':{
          const destinationBuffer = t.createBufferWithState('valid', {
            size: 4,
            usage: GPUBufferUsage.COPY_DST
          });
          encoder.copyBufferToBuffer(buffer, 0, destinationBuffer, 0, 4);
          break;
        }
      case 'copy-dst':{
          const sourceBuffer = t.createBufferWithState('valid', {
            size: 4,
            usage: GPUBufferUsage.COPY_SRC
          });
          encoder.copyBufferToBuffer(sourceBuffer, 0, buffer, 0, 4);
          break;
        }
      case 'uniform':
      case 'storage':
      case 'read-only-storage':{
          const bindGroup = t.createBindGroupForTest(buffer, 0, usage, 'fragment');
          switch (pass) {
            case 'render':{
                const renderPassEncoder = t.beginSimpleRenderPass(encoder);
                renderPassEncoder.setBindGroup(0, bindGroup);
                renderPassEncoder.end();
                break;
              }
            case 'compute':{
                const computePassEncoder = encoder.beginComputePass();
                computePassEncoder.setBindGroup(0, bindGroup);
                computePassEncoder.end();
                break;
              }
            default:
              unreachable();
          }
          break;
        }
      case 'vertex':{
          const renderPassEncoder = t.beginSimpleRenderPass(encoder);
          renderPassEncoder.setVertexBuffer(0, buffer);
          renderPassEncoder.end();
          break;
        }
      case 'index':{
          const renderPassEncoder = t.beginSimpleRenderPass(encoder);
          renderPassEncoder.setIndexBuffer(buffer, 'uint16');
          renderPassEncoder.end();
          break;
        }
      case 'indirect':{
          switch (pass) {
            case 'render':{
                const renderPassEncoder = t.beginSimpleRenderPass(encoder);
                const renderPipeline = t.createNoOpRenderPipeline();
                renderPassEncoder.setPipeline(renderPipeline);
                renderPassEncoder.drawIndirect(buffer, 0);
                renderPassEncoder.end();
                break;
              }
            case 'compute':{
                const computePassEncoder = encoder.beginComputePass();
                const computePipeline = t.createNoOpComputePipeline();
                computePassEncoder.setPipeline(computePipeline);
                computePassEncoder.dispatchWorkgroupsIndirect(buffer, 0);
                computePassEncoder.end();
                break;
              }
            default:
              unreachable();
          }
          break;
        }
      case 'indexedIndirect':{
          const renderPassEncoder = t.beginSimpleRenderPass(encoder);
          const renderPipeline = t.createNoOpRenderPipeline();
          renderPassEncoder.setPipeline(renderPipeline);
          const indexBuffer = t.createBufferWithState('valid', {
            size: 4,
            usage: GPUBufferUsage.INDEX
          });
          renderPassEncoder.setIndexBuffer(indexBuffer, 'uint16');
          renderPassEncoder.drawIndexedIndirect(buffer, 0);
          renderPassEncoder.end();
          break;
        }
      default:
        unreachable();
    }
  };

  const encoder = t.device.createCommandEncoder();
  UseBufferOnCommandEncoder(usage0, encoder);
  UseBufferOnCommandEncoder(usage1, encoder);
  t.expectValidationError(() => {
    encoder.finish();
  }, false);
});