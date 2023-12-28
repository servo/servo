/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for the map-state of mappable buffers used in submitted command buffers.

Tests every operation that has a dependency on a buffer
  - writeBuffer
  - copyB2B {src,dst}
  - copyB2T
  - copyT2B

Test those operations against buffers in the following states:
  - Unmapped
  - In the process of mapping
  - mapped
  - mapped with a mapped range queried
  - unmapped after mapping
  - mapped at creation

Also tests every order of operations combination of mapping operations and command recording
operations to ensure the mapping state is only considered when a command buffer is submitted.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ValidationTest } from '../validation_test.js';

class F extends ValidationTest {
  async runBufferDependencyTest(usage, callback) {
    const bufferDesc = {
      size: 8,
      usage,
      mappedAtCreation: false
    };

    const mapMode = usage & GPUBufferUsage.MAP_READ ? GPUMapMode.READ : GPUMapMode.WRITE;

    // Create a mappable buffer, and one that will remain unmapped for comparison.
    const mappableBuffer = this.device.createBuffer(bufferDesc);
    const unmappedBuffer = this.device.createBuffer(bufferDesc);

    // Run the given operation before the buffer is mapped. Should succeed.
    callback(mappableBuffer);

    // Map the buffer
    const mapPromise = mappableBuffer.mapAsync(mapMode);

    // Run the given operation while the buffer is in the process of mapping. Should fail.
    this.expectValidationError(() => {
      callback(mappableBuffer);
    });

    // Run on a different, unmapped buffer. Should succeed.
    callback(unmappedBuffer);

    await mapPromise;

    // Run the given operation when the buffer is finished mapping with no getMappedRange. Should fail.
    this.expectValidationError(() => {
      callback(mappableBuffer);
    });

    // Run on a different, unmapped buffer. Should succeed.
    callback(unmappedBuffer);

    // Run the given operation when the buffer is mapped with getMappedRange. Should fail.
    mappableBuffer.getMappedRange();
    this.expectValidationError(() => {
      callback(mappableBuffer);
    });

    // Unmap the buffer and run the operation. Should succeed.
    mappableBuffer.unmap();
    callback(mappableBuffer);

    // Create a buffer that's mappedAtCreation.
    bufferDesc.mappedAtCreation = true;
    const mappedBuffer = this.device.createBuffer(bufferDesc);

    // Run the operation with the mappedAtCreation buffer. Should fail.
    this.expectValidationError(() => {
      callback(mappedBuffer);
    });

    // Run on a different, unmapped buffer. Should succeed.
    callback(unmappedBuffer);

    // Unmap the mappedAtCreation buffer and run the operation. Should succeed.
    mappedBuffer.unmap();
    callback(mappedBuffer);
  }
}

export const g = makeTestGroup(F);

g.test('writeBuffer').
desc(`Test that an outstanding mapping will prevent writeBuffer calls.`).
fn(async (t) => {
  const data = new Uint32Array([42]);

  await t.runBufferDependencyTest(
    GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST,
    (buffer) => {
      t.queue.writeBuffer(buffer, 0, data);
    }
  );
});

g.test('copyBufferToBuffer').
desc(
  `
  Test that an outstanding mapping will prevent copyBufferToTexture commands from submitting,
  both when used as the source and destination.`
).
fn(async (t) => {
  const sourceBuffer = t.device.createBuffer({
    size: 8,
    usage: GPUBufferUsage.COPY_SRC
  });

  const destBuffer = t.device.createBuffer({
    size: 8,
    usage: GPUBufferUsage.COPY_DST
  });

  await t.runBufferDependencyTest(
    GPUBufferUsage.MAP_WRITE | GPUBufferUsage.COPY_SRC,
    (buffer) => {
      const commandEncoder = t.device.createCommandEncoder();
      commandEncoder.copyBufferToBuffer(buffer, 0, destBuffer, 0, 4);
      t.queue.submit([commandEncoder.finish()]);
    }
  );

  await t.runBufferDependencyTest(
    GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST,
    (buffer) => {
      const commandEncoder = t.device.createCommandEncoder();
      commandEncoder.copyBufferToBuffer(sourceBuffer, 0, buffer, 0, 4);
      t.queue.submit([commandEncoder.finish()]);
    }
  );
});

g.test('copyBufferToTexture').
desc(
  `Test that an outstanding mapping will prevent copyBufferToTexture commands from submitting.`
).
fn(async (t) => {
  const size = { width: 1, height: 1 };

  const texture = t.device.createTexture({
    size,
    format: 'rgba8unorm',
    usage: GPUTextureUsage.COPY_DST
  });

  await t.runBufferDependencyTest(
    GPUBufferUsage.MAP_WRITE | GPUBufferUsage.COPY_SRC,
    (buffer) => {
      const commandEncoder = t.device.createCommandEncoder();
      commandEncoder.copyBufferToTexture({ buffer }, { texture }, size);
      t.queue.submit([commandEncoder.finish()]);
    }
  );
});

g.test('copyTextureToBuffer').
desc(
  `Test that an outstanding mapping will prevent copyTextureToBuffer commands from submitting.`
).
fn(async (t) => {
  const size = { width: 1, height: 1 };

  const texture = t.device.createTexture({
    size,
    format: 'rgba8unorm',
    usage: GPUTextureUsage.COPY_SRC
  });

  await t.runBufferDependencyTest(
    GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST,
    (buffer) => {
      const commandEncoder = t.device.createCommandEncoder();
      commandEncoder.copyTextureToBuffer({ texture }, { buffer }, size);
      t.queue.submit([commandEncoder.finish()]);
    }
  );
});

g.test('map_command_recording_order').
desc(
  `
Test that the order of mapping a buffer relative to when commands are recorded that use it
  does not matter, as long as the buffer is unmapped when the commands are submitted.
  `
).
paramsSubcasesOnly([
{
  order: ['record', 'map', 'unmap', 'finish', 'submit'],
  mappedAtCreation: false,
  _shouldError: false
},
{
  order: ['record', 'map', 'finish', 'unmap', 'submit'],
  mappedAtCreation: false,
  _shouldError: false
},
{
  order: ['record', 'finish', 'map', 'unmap', 'submit'],
  mappedAtCreation: false,
  _shouldError: false
},
{
  order: ['map', 'record', 'unmap', 'finish', 'submit'],
  mappedAtCreation: false,
  _shouldError: false
},
{
  order: ['map', 'record', 'finish', 'unmap', 'submit'],
  mappedAtCreation: false,
  _shouldError: false
},
{
  order: ['map', 'record', 'finish', 'submit', 'unmap'],
  mappedAtCreation: false,
  _shouldError: true
},
{
  order: ['record', 'map', 'finish', 'submit', 'unmap'],
  mappedAtCreation: false,
  _shouldError: true
},
{
  order: ['record', 'finish', 'map', 'submit', 'unmap'],
  mappedAtCreation: false,
  _shouldError: true
},
{ order: ['record', 'unmap', 'finish', 'submit'], mappedAtCreation: true, _shouldError: false },
{ order: ['record', 'finish', 'unmap', 'submit'], mappedAtCreation: true, _shouldError: false },
{ order: ['record', 'finish', 'submit', 'unmap'], mappedAtCreation: true, _shouldError: true }]
).
fn(async (t) => {
  const { order, mappedAtCreation, _shouldError: shouldError } = t.params;

  const buffer = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.MAP_WRITE | GPUBufferUsage.COPY_SRC,
    mappedAtCreation
  });

  const targetBuffer = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.COPY_DST
  });

  const commandEncoder = t.device.createCommandEncoder();
  let commandBuffer;

  const steps = {
    record: () => {
      commandEncoder.copyBufferToBuffer(buffer, 0, targetBuffer, 0, 4);
    },
    map: async () => {
      await buffer.mapAsync(GPUMapMode.WRITE);
    },
    unmap: () => {
      buffer.unmap();
    },
    finish: () => {
      commandBuffer = commandEncoder.finish();
    },
    submit: () => {
      t.expectValidationError(() => {
        t.queue.submit([commandBuffer]);
      }, shouldError);
    }
  };

  for (const op of order) {
    await steps[op]();
  }
});