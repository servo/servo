/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
API operations tests for timestamp queries.

Given the values returned are implementation defined
there is not much we can test except that there are no errors.

- test query with
  - compute pass
  - render pass
  - 64k query objects
  - resolving unused slots
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { range } from '../../../../../common/util/util.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../../gpu_test.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

function encodeTimestampQueries(
encoder,
view,
querySet,
stage,
slot)
{
  switch (stage) {
    case 'compute':{
        const pass = encoder.beginComputePass({
          timestampWrites: {
            querySet,
            beginningOfPassWriteIndex: slot,
            endOfPassWriteIndex: slot + 1
          }
        });
        pass.end();
        break;
      }
    case 'render':{
        const pass = encoder.beginRenderPass({
          colorAttachments: [{ view, loadOp: 'load', storeOp: 'store' }],
          timestampWrites: {
            querySet,
            beginningOfPassWriteIndex: slot,
            endOfPassWriteIndex: slot + 1
          }
        });
        pass.end();
        break;
      }
  }
}

g.test('many_query_sets').
desc(
  `
Test creating and using 64k query objects.

This test is because there is a Metal limit of 32 MTLCounterSampleBuffers
Implementations are supposed to work around this limit by internally allocating
larger MTLCounterSampleBuffers and having the WebGPU sets be subsets of those
larger buffers.

This is particular important as the limit is 32 per process
so a few pages making a few queries would easily hit the limit
and prevent pages from running.
    `
).
params((u) =>
u.
combine('numQuerySets', [8, 16, 32, 64, 256, 65536]).
combine('stage', ['compute', 'render'])
).
fn(async (t) => {
  const { stage, numQuerySets } = t.params;

  t.skipIfDeviceDoesNotHaveFeature('timestamp-query');

  // At large numQuerySets, this test incurs a lot of validation, which can take several seconds.
  // Explicitly wrap the test in its own error scope to avoid triggering timeouts in test-cleanup.
  t.device.pushErrorScope('validation');
  try {
    const view = t.
    createTextureTracked({
      size: [1, 1, 1],
      format: 'rgba8unorm',
      usage: GPUTextureUsage.RENDER_ATTACHMENT
    }).
    createView();
    const encoder = t.device.createCommandEncoder();

    for (let i = 0; i < numQuerySets; ++i) {
      const querySet = t.createQuerySetTracked({
        type: 'timestamp',
        count: 2
      });

      encodeTimestampQueries(encoder, view, querySet, stage, 0);
    }

    const shouldError = false; // just expect no error
    t.expectValidationError(() => t.device.queue.submit([encoder.finish()]), shouldError);
  } finally {
    const error = await t.device.popErrorScope();
    // Make sure there weren't any unexpected validation errors caught by the scope.
    t.expect(error === null, error?.message);
  }
});

function encoderQueryUsage(
t,
stage,
numQuerySets,
numSlots)
{
  const encoder = t.device.createCommandEncoder();

  const view = t.
  createTextureTracked({
    size: [1, 1, 1],
    format: 'rgba8unorm',
    usage: GPUTextureUsage.RENDER_ATTACHMENT
  }).
  createView();

  const querySets = range(numQuerySets, (_) => {
    const querySet = t.createQuerySetTracked({
      type: 'timestamp',
      count: numSlots
    });

    for (let slot = 0; slot < numSlots; slot += 2) {
      encodeTimestampQueries(encoder, view, querySet, stage, slot);
    }

    return querySet;
  });
  return { encoder, querySets };
}

g.test('many_slots').
desc(
  `
Test creating and using 4k query slots.

Metal has the limit that a MTLCounterSampleBuffer can be max 32k which is 4k slots.
So, test we can use 4k slots across a few QuerySets
    `
).
params((u) => u.combine('stage', ['compute', 'render'])).
fn((t) => {
  const { stage } = t.params;

  t.skipIfDeviceDoesNotHaveFeature('timestamp-query');
  const kNumSlots = 4096;
  const kNumQuerySets = 4;

  const { encoder } = encoderQueryUsage(t, stage, kNumQuerySets, kNumSlots);
  t.device.queue.submit([encoder.finish()]);
});

g.test('resolve_unused_slots').
desc(
  `
Test resolving query sets with unused slots.

We create a command buffer that uses the slots but don't actually submit it
to make sure the implementation doesn't mistakenly mark them as used.
    `
).
params((u) => u.combine('stage', ['compute', 'render'])).
fn((t) => {
  const { stage } = t.params;

  t.skipIfDeviceDoesNotHaveFeature('timestamp-query');

  const kNumSlots = 4096;
  const kNumQuerySets = 2;

  // Create a encoder and encode usage of every query and slot but do not submit it.
  const querySets = (() => {
    const { encoder, querySets } = encoderQueryUsage(t, stage, kNumQuerySets, kNumSlots);
    encoder.finish();
    return querySets;
  })();

  // Read the slots, they should all be zero.
  const encoder = t.device.createCommandEncoder();
  const buffers = querySets.map((querySet, i) => {
    const resolveBuffer = t.createBufferTracked({
      size: kNumSlots * 8,
      usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.QUERY_RESOLVE
    });
    encoder.resolveQuerySet(querySet, 0, kNumSlots, resolveBuffer, 0);
    return resolveBuffer;
  });
  t.device.queue.submit([encoder.finish()]);

  for (const buffer of buffers) {
    const expected = new Uint8Array(buffer.size);
    t.expectGPUBufferValuesEqual(buffer, expected);
  }
});

g.test('multi_resolve').
desc(`Test resolving more than once does not change the results`).
params((u) => u.combine('stage', ['compute', 'render'])).
fn(async (t) => {
  const { stage } = t.params;
  const kNumQueries = 64;

  t.skipIfDeviceDoesNotHaveFeature('timestamp-query');

  const querySet = t.createQuerySetTracked({
    type: 'timestamp',
    count: kNumQueries
  });

  const view = t.
  createTextureTracked({
    size: [1, 1, 1],
    format: 'rgba8unorm',
    usage: GPUTextureUsage.RENDER_ATTACHMENT
  }).
  createView();
  const encoder = t.device.createCommandEncoder();

  for (let slot = 0; slot < kNumQueries; slot += 2) {
    // skip every other pair so we can test resolving un-used slots
    const pair = slot / 2 | 0;
    if (pair % 2) {
      continue;
    }
    encodeTimestampQueries(encoder, view, querySet, stage, slot);
  }

  const size = kNumQueries * 8;
  const resolveBuffer1 = t.createBufferTracked({
    size,
    usage: GPUBufferUsage.QUERY_RESOLVE | GPUBufferUsage.COPY_SRC
  });
  const resolveBuffer2 = t.createBufferTracked({
    size,
    usage: GPUBufferUsage.QUERY_RESOLVE | GPUBufferUsage.COPY_SRC
  });
  const resultBuffer1 = t.createBufferTracked({
    size,
    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ
  });

  encoder.resolveQuerySet(querySet, 0, kNumQueries, resolveBuffer1, 0);
  encoder.resolveQuerySet(querySet, 0, kNumQueries, resolveBuffer2, 0);
  encoder.copyBufferToBuffer(resolveBuffer1, 0, resultBuffer1, 0, size);

  t.device.queue.submit([encoder.finish()]);

  // Read back the first result.
  await resultBuffer1.mapAsync(GPUMapMode.READ);
  const expected = new Uint32Array(resultBuffer1.getMappedRange());
  t.expectGPUBufferValuesEqual(resolveBuffer2, expected);
});

g.test('unused_slots_are_zero').
desc(`Test that unused slots are resolved to zero`).
params((u) => u.combine('stage', ['compute', 'render'])).
fn((t) => {
  const { stage } = t.params;
  const kNumQueries = 64;

  t.skipIfDeviceDoesNotHaveFeature('timestamp-query');

  const querySet = t.createQuerySetTracked({
    type: 'timestamp',
    count: kNumQueries
  });

  const view = t.
  createTextureTracked({
    size: [1, 1, 1],
    format: 'rgba8unorm',
    usage: GPUTextureUsage.RENDER_ATTACHMENT
  }).
  createView();
  const usedEncoder = t.device.createCommandEncoder();
  const unusedEncoder = t.device.createCommandEncoder();

  for (let slot = 0; slot < kNumQueries; slot += 2) {
    const pair = slot / 2 | 0;
    const encoder = pair % 2 ? usedEncoder : unusedEncoder;
    encodeTimestampQueries(encoder, view, querySet, stage, slot);
  }

  unusedEncoder.finish(); // don't submit this encoder

  const size = kNumQueries * 8;
  const resolveBuffer = t.createBufferTracked({
    size,
    usage: GPUBufferUsage.QUERY_RESOLVE | GPUBufferUsage.COPY_SRC
  });

  usedEncoder.resolveQuerySet(querySet, 0, kNumQueries, resolveBuffer, 0);

  t.device.queue.submit([usedEncoder.finish()]);

  // Read back the first result.
  t.expectGPUBufferValuesPassCheck(
    resolveBuffer,
    (actualU32) => {
      // MAINTENANCE_TODO: expectGPUBufferValuesPassCheck doesn't work with BigUint64Array.
      const actual = new BigUint64Array(
        actualU32.buffer,
        actualU32.byteOffset,
        actualU32.byteLength / 8
      );
      const errors = [];
      for (let slot = 0; slot < kNumQueries; ++slot) {
        t.debug(() => `slot ${slot}: ${actual[slot]}`);
        const pair = slot / 2 | 0;
        if (pair % 2) {

          // used slot, implementation defined so don't check
        } else {// unused slot, expect zero
          if (actual[slot] !== 0n) {
            errors.push(`slot ${slot} expected 0 but got ${actual[slot]}`);
          }
        }
      }
      return errors.length === 0 ? undefined : new Error(errors.join('\n'));
    },
    {
      type: Uint32Array,
      typedLength: kNumQueries * 2 // because it's really BigUint64Array data
    }
  );
});