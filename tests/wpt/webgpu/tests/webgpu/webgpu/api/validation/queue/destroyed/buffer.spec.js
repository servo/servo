/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests using a destroyed buffer on a queue.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { ValidationTest } from '../../validation_test.js';

export const g = makeTestGroup(ValidationTest);

g.test('writeBuffer').
desc(
  `
Tests that using a destroyed buffer in writeBuffer fails.
- x= {destroyed, not destroyed (control case)}
  `
).
paramsSubcasesOnly((u) => u.combine('destroyed', [false, true])).
fn((t) => {
  const { destroyed } = t.params;
  const buffer = t.trackForCleanup(
    t.device.createBuffer({
      size: 4,
      usage: GPUBufferUsage.COPY_DST
    })
  );

  if (destroyed) {
    buffer.destroy();
  }

  t.expectValidationError(() => t.queue.writeBuffer(buffer, 0, new Uint8Array(4)), destroyed);
});

g.test('copyBufferToBuffer').
desc(
  `
Tests that using a destroyed buffer in copyBufferToBuffer fails.
- x= {not destroyed (control case), src destroyed, dst destroyed}
  `
).
paramsSubcasesOnly((u) => u.combine('destroyed', ['none', 'src', 'dst', 'both'])).
fn((t) => {
  const src = t.trackForCleanup(
    t.device.createBuffer({ size: 4, usage: GPUBufferUsage.COPY_SRC })
  );
  const dst = t.trackForCleanup(
    t.device.createBuffer({ size: 4, usage: GPUBufferUsage.COPY_DST })
  );

  const encoder = t.device.createCommandEncoder();
  encoder.copyBufferToBuffer(src, 0, dst, 0, dst.size);
  const commandBuffer = encoder.finish();

  let shouldError = true;
  switch (t.params.destroyed) {
    case 'none':
      shouldError = false;
      break;
    case 'src':
      src.destroy();
      break;
    case 'dst':
      dst.destroy();
      break;
    case 'both':
      src.destroy();
      dst.destroy();
      break;
  }

  t.expectValidationError(() => {
    t.queue.submit([commandBuffer]);
  }, shouldError);
});

g.test('copyBufferToTexture').
desc(
  `
Tests that using a destroyed buffer in copyBufferToTexture fails.
- x= {not destroyed (control case), src destroyed}
  `
).
paramsSubcasesOnly((u) => u.combine('destroyed', [false, true])).
fn((t) => {
  const { destroyed } = t.params;
  const buffer = t.trackForCleanup(
    t.device.createBuffer({ size: 4, usage: GPUBufferUsage.COPY_SRC })
  );
  const texture = t.trackForCleanup(
    t.device.createTexture({
      size: [1, 1, 1],
      format: 'rgba8unorm',
      usage: GPUTextureUsage.COPY_DST
    })
  );

  const encoder = t.device.createCommandEncoder();
  encoder.copyBufferToTexture({ buffer }, { texture }, [1, 1, 1]);
  const commandBuffer = encoder.finish();

  if (destroyed) {
    buffer.destroy();
  }

  t.expectValidationError(() => {
    t.queue.submit([commandBuffer]);
  }, destroyed);
});

g.test('copyTextureToBuffer').
desc(
  `
Tests that using a destroyed buffer in copyTextureToBuffer fails.
- x= {not destroyed (control case), dst destroyed}
  `
).
paramsSubcasesOnly((u) => u.combine('destroyed', [false, true])).
fn((t) => {
  const { destroyed } = t.params;
  const texture = t.trackForCleanup(
    t.device.createTexture({
      size: [1, 1, 1],
      format: 'rgba8unorm',
      usage: GPUTextureUsage.COPY_SRC
    })
  );
  const buffer = t.trackForCleanup(
    t.device.createBuffer({ size: 4, usage: GPUBufferUsage.COPY_DST })
  );

  const encoder = t.device.createCommandEncoder();
  encoder.copyTextureToBuffer({ texture }, { buffer }, [1, 1, 1]);
  const commandBuffer = encoder.finish();

  if (destroyed) {
    buffer.destroy();
  }

  t.expectValidationError(() => {
    t.queue.submit([commandBuffer]);
  }, destroyed);
});

g.test('setBindGroup').
desc(
  `
Tests that using a destroyed buffer referenced by a bindGroup set with setBindGroup fails
- x= {not destroyed (control case), destroyed}
    `
).
paramsSubcasesOnly((u) =>
u.
combine('destroyed', [false, true]).
combine('encoderType', ['compute pass', 'render pass', 'render bundle'])
).
fn((t) => {
  const { destroyed, encoderType } = t.params;
  const { device } = t;
  const buffer = t.trackForCleanup(
    t.device.createBuffer({
      size: 4,
      usage: GPUBufferUsage.UNIFORM
    })
  );

  const layout = device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUShaderStage.COMPUTE | GPUShaderStage.VERTEX,
      buffer: {}
    }]

  });

  const bindGroup = device.createBindGroup({
    layout,
    entries: [{ binding: 0, resource: { buffer } }]
  });

  const { encoder, finish } = t.createEncoder(encoderType);
  encoder.setBindGroup(0, bindGroup);
  const commandBuffer = finish();

  if (destroyed) {
    buffer.destroy();
  }

  t.expectValidationError(() => {
    t.queue.submit([commandBuffer]);
  }, destroyed);
});

g.test('setVertexBuffer').
desc(
  `
Tests that using a destroyed buffer referenced in a render pass fails
- x= {not destroyed (control case), destroyed}
    `
).
paramsSubcasesOnly((u) =>
u.
combine('destroyed', [false, true]).
combine('encoderType', ['render pass', 'render bundle'])
).
fn((t) => {
  const { destroyed, encoderType } = t.params;
  const vertexBuffer = t.trackForCleanup(
    t.device.createBuffer({
      size: 4,
      usage: GPUBufferUsage.VERTEX
    })
  );

  const { encoder, finish } = t.createEncoder(encoderType);
  encoder.setVertexBuffer(0, vertexBuffer);
  const commandBuffer = finish();

  if (destroyed) {
    vertexBuffer.destroy();
  }

  t.expectValidationError(() => {
    t.queue.submit([commandBuffer]);
  }, destroyed);
});

g.test('setIndexBuffer').
desc(
  `
Tests that using a destroyed buffer referenced in a render pass fails
- x= {not destroyed (control case), destroyed}
    `
).
paramsSubcasesOnly((u) =>
u.
combine('destroyed', [false, true]).
combine('encoderType', ['render pass', 'render bundle'])
).
fn((t) => {
  const { destroyed, encoderType } = t.params;
  const indexBuffer = t.trackForCleanup(
    t.device.createBuffer({
      size: 4,
      usage: GPUBufferUsage.INDEX
    })
  );

  const { encoder, finish } = t.createEncoder(encoderType);
  encoder.setIndexBuffer(indexBuffer, 'uint16');
  const commandBuffer = finish();

  if (destroyed) {
    indexBuffer.destroy();
  }

  t.expectValidationError(() => {
    t.queue.submit([commandBuffer]);
  }, destroyed);
});

g.test('resolveQuerySet').
desc(
  `
Tests that using a destroyed buffer referenced via resolveQuerySet fails
- x= {not destroyed (control case), destroyed}
    `
).
paramsSubcasesOnly((u) => u.combine('destroyed', [false, true])).
fn((t) => {
  const { destroyed } = t.params;
  const querySet = t.trackForCleanup(
    t.device.createQuerySet({
      type: 'occlusion',
      count: 1
    })
  );
  const querySetBuffer = t.trackForCleanup(
    t.device.createBuffer({
      size: 8,
      usage: GPUBufferUsage.QUERY_RESOLVE
    })
  );

  const encoder = t.device.createCommandEncoder();
  encoder.resolveQuerySet(querySet, 0, 1, querySetBuffer, 0);
  const commandBuffer = encoder.finish();

  if (destroyed) {
    querySetBuffer.destroy();
  }

  t.expectValidationError(() => {
    t.queue.submit([commandBuffer]);
  }, destroyed);
});