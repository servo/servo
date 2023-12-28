/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for GPUBuffer.mapAsync, GPUBuffer.unmap and GPUBuffer.getMappedRange.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { attemptGarbageCollection } from '../../../../common/util/collect_garbage.js';
import { assert, unreachable } from '../../../../common/util/util.js';
import { kBufferUsages } from '../../../capability_info.js';
import { GPUConst } from '../../../constants.js';
import { ValidationTest } from '../validation_test.js';

class F extends ValidationTest {
  async testMapAsyncCall(
  expectation,


  buffer,
  mode,
  offset,
  size)
  {
    if (expectation === 'success') {
      const p = buffer.mapAsync(mode, offset, size);
      await p;
    } else {
      let p;
      this.expectValidationError(() => {
        p = buffer.mapAsync(mode, offset, size);
      }, expectation.validationError);
      let caught = false;
      let rejectedEarly = false;
      // If mapAsync rejected early, microtask A will run before B.
      // If not, B will run before A.
      p.catch(() => {
        // Microtask A
        caught = true;
      });
      queueMicrotask(() => {
        // Microtask B
        rejectedEarly = caught;
      });
      try {
        // This await will always complete after microtasks A and B are both done.
        await p;
        assert(expectation.rejectName === null, 'mapAsync unexpectedly passed');
      } catch (ex) {
        assert(ex instanceof Error, 'mapAsync rejected with non-error');
        assert(typeof ex.stack === 'string', 'mapAsync rejected without a stack');
        assert(expectation.rejectName === ex.name, `mapAsync rejected unexpectedly with: ${ex}`);
        assert(
          expectation.earlyRejection === rejectedEarly,
          'mapAsync rejected at an unexpected timing'
        );
      }
    }
  }

  testGetMappedRangeCall(success, buffer, offset, size) {
    if (success) {
      const data = buffer.getMappedRange(offset, size);
      this.expect(data instanceof ArrayBuffer);
      if (size !== undefined) {
        this.expect(data.byteLength === size);
      }
    } else {
      this.shouldThrow('OperationError', () => {
        buffer.getMappedRange(offset, size);
      });
    }
  }

  createMappableBuffer(type, size) {
    switch (type) {
      case GPUMapMode.READ:
        return this.device.createBuffer({
          size,
          usage: GPUBufferUsage.MAP_READ
        });
      case GPUMapMode.WRITE:
        return this.device.createBuffer({
          size,
          usage: GPUBufferUsage.MAP_WRITE
        });
      default:
        unreachable();
    }
  }
}

export const g = makeTestGroup(F);

const kMapModeOptions = [GPUConst.MapMode.READ, GPUConst.MapMode.WRITE];
const kOffsetAlignment = 8;
const kSizeAlignment = 4;

g.test('mapAsync,usage').
desc(
  `Test the usage validation for mapAsync.

  For each buffer usage:
  For GPUMapMode.READ, GPUMapMode.WRITE, and 0:
    Test that the mapAsync call is valid iff the mapping usage is not 0 and the buffer usage
    the mapMode flag.`
).
paramsSubcasesOnly((u) =>
u //
.combineWithParams([
{ mapMode: GPUConst.MapMode.READ, validUsage: GPUConst.BufferUsage.MAP_READ },
{ mapMode: GPUConst.MapMode.WRITE, validUsage: GPUConst.BufferUsage.MAP_WRITE },
// Using mapMode 0 is never valid, so there is no validUsage.
{ mapMode: 0, validUsage: null }]
).
combine('usage', kBufferUsages)
).
fn(async (t) => {
  const { mapMode, validUsage, usage } = t.params;

  const buffer = t.device.createBuffer({
    size: 16,
    usage
  });

  const successParam =
  usage === validUsage ?
  'success' :
  {
    validationError: true,
    earlyRejection: false,
    rejectName: 'OperationError'
  };
  await t.testMapAsyncCall(successParam, buffer, mapMode);
});

g.test('mapAsync,invalidBuffer').
desc('Test that mapAsync is an error when called on an invalid buffer.').
paramsSubcasesOnly((u) => u.combine('mapMode', kMapModeOptions)).
fn(async (t) => {
  const { mapMode } = t.params;
  const buffer = t.getErrorBuffer();
  await t.testMapAsyncCall(
    { validationError: true, earlyRejection: false, rejectName: 'OperationError' },
    buffer,
    mapMode
  );
});

g.test('mapAsync,state,destroyed').
desc('Test that mapAsync is an error when called on a destroyed buffer.').
paramsSubcasesOnly((u) => u.combine('mapMode', kMapModeOptions)).
fn(async (t) => {
  const { mapMode } = t.params;
  const buffer = t.createMappableBuffer(mapMode, 16);

  // Start mapping the buffer, we are going to destroy it before it resolves so it will reject
  // the mapping promise with an AbortError.
  const pending = t.testMapAsyncCall(
    { validationError: false, earlyRejection: false, rejectName: 'AbortError' },
    buffer,
    mapMode
  );

  buffer.destroy();
  await t.testMapAsyncCall(
    { validationError: true, earlyRejection: false, rejectName: 'OperationError' },
    buffer,
    mapMode
  );

  await pending;
});

g.test('mapAsync,state,mappedAtCreation').
desc(
  `Test that mapAsync is an error when called on a buffer mapped at creation,
    but succeeds after unmapping it.`
).
paramsSubcasesOnly([
{ mapMode: GPUConst.MapMode.READ, validUsage: GPUConst.BufferUsage.MAP_READ },
{ mapMode: GPUConst.MapMode.WRITE, validUsage: GPUConst.BufferUsage.MAP_WRITE }]
).
fn(async (t) => {
  const { mapMode, validUsage } = t.params;

  const buffer = t.device.createBuffer({
    size: 16,
    usage: validUsage,
    mappedAtCreation: true
  });
  await t.testMapAsyncCall(
    { validationError: true, earlyRejection: false, rejectName: 'OperationError' },
    buffer,
    mapMode
  );

  buffer.unmap();
  await t.testMapAsyncCall('success', buffer, mapMode);
});

g.test('mapAsync,state,mapped').
desc(
  `Test that mapAsync is an error when called on a mapped buffer, but succeeds
    after unmapping it.`
).
paramsSubcasesOnly((u) => u.combine('mapMode', kMapModeOptions)).
fn(async (t) => {
  const { mapMode } = t.params;

  const buffer = t.createMappableBuffer(mapMode, 16);
  await t.testMapAsyncCall('success', buffer, mapMode);
  await t.testMapAsyncCall(
    { validationError: true, earlyRejection: false, rejectName: 'OperationError' },
    buffer,
    mapMode
  );

  buffer.unmap();
  await t.testMapAsyncCall('success', buffer, mapMode);
});

g.test('mapAsync,state,mappingPending').
desc(
  `Test that mapAsync is rejected when called on a buffer that is being mapped,
    but succeeds after the previous mapping request is cancelled.`
).
paramsSubcasesOnly((u) => u.combine('mapMode', kMapModeOptions)).
fn(async (t) => {
  const { mapMode } = t.params;

  const buffer = t.createMappableBuffer(mapMode, 16);

  // Start mapping the buffer, we are going to unmap it before it resolves so it will reject
  // the mapping promise with an AbortError.
  const pending0 = t.testMapAsyncCall(
    { validationError: false, earlyRejection: false, rejectName: 'AbortError' },
    buffer,
    mapMode
  );

  // Do the test of mapAsync while [[pending_map]] is non-null. It has to be synchronous so
  // that we can unmap the previous mapping in the same stack frame and testing this one doesn't
  // get canceled, but instead is rejected.
  const pending1 = t.testMapAsyncCall(
    { validationError: false, earlyRejection: true, rejectName: 'OperationError' },
    buffer,
    mapMode
  );

  // Unmap the first mapping. It should now be possible to successfully call mapAsync
  // This unmap should cause the first mapAsync rejection.
  buffer.unmap();
  await t.testMapAsyncCall('success', buffer, mapMode);

  await pending0;
  await pending1;
});

g.test('mapAsync,sizeUnspecifiedOOB').
desc(
  `Test that mapAsync with size unspecified rejects if offset > buffer.[[size]],
    with various cases at the limits of the buffer size or with a misaligned offset.
    Also test for an empty buffer.`
).
paramsSubcasesOnly((u) =>
u //
.combine('mapMode', kMapModeOptions).
combineWithParams([
// 0 size buffer.
{ bufferSize: 0, offset: 0 },
{ bufferSize: 0, offset: 1 },
{ bufferSize: 0, offset: kOffsetAlignment },

// Test with a buffer that's not empty.
{ bufferSize: 16, offset: 0 },
{ bufferSize: 16, offset: kOffsetAlignment },
{ bufferSize: 16, offset: 16 },
{ bufferSize: 16, offset: 17 },
{ bufferSize: 16, offset: 16 + kOffsetAlignment }]
)
).
fn(async (t) => {
  const { mapMode, bufferSize, offset } = t.params;
  const buffer = t.createMappableBuffer(mapMode, bufferSize);

  const successParam =
  offset <= bufferSize ?
  'success' :
  {
    validationError: true,
    earlyRejection: false,
    rejectName: 'OperationError'
  };
  await t.testMapAsyncCall(successParam, buffer, mapMode, offset);
});

g.test('mapAsync,offsetAndSizeAlignment').
desc("Test that mapAsync fails if the alignment of offset and size isn't correct.").
paramsSubcasesOnly((u) =>
u.
combine('mapMode', kMapModeOptions).
combine('offset', [0, kOffsetAlignment, kOffsetAlignment / 2]).
combine('size', [0, kSizeAlignment, kSizeAlignment / 2])
).
fn(async (t) => {
  const { mapMode, offset, size } = t.params;
  const buffer = t.createMappableBuffer(mapMode, 16);

  const successParam =
  offset % kOffsetAlignment === 0 && size % kSizeAlignment === 0 ?
  'success' :
  {
    validationError: true,
    earlyRejection: false,
    rejectName: 'OperationError'
  };
  await t.testMapAsyncCall(successParam, buffer, mapMode, offset, size);
});

g.test('mapAsync,offsetAndSizeOOB').
desc('Test that mapAsync fails if offset + size is larger than the buffer size.').
paramsSubcasesOnly((u) =>
u //
.combine('mapMode', kMapModeOptions).
combineWithParams([
// For a 0 size buffer
{ bufferSize: 0, offset: 0, size: 0 },
{ bufferSize: 0, offset: 0, size: 4 },
{ bufferSize: 0, offset: 8, size: 0 },

// For a small buffer
{ bufferSize: 16, offset: 0, size: 16 },
{ bufferSize: 16, offset: kOffsetAlignment, size: 16 },

{ bufferSize: 16, offset: 16, size: 0 },
{ bufferSize: 16, offset: 16, size: kSizeAlignment },

{ bufferSize: 16, offset: 8, size: 0 },
{ bufferSize: 16, offset: 8, size: 8 },
{ bufferSize: 16, offset: 8, size: 8 + kSizeAlignment },

// For a larger buffer
{ bufferSize: 1024, offset: 0, size: 1024 },
{ bufferSize: 1024, offset: kOffsetAlignment, size: 1024 },

{ bufferSize: 1024, offset: 1024, size: 0 },
{ bufferSize: 1024, offset: 1024, size: kSizeAlignment },

{ bufferSize: 1024, offset: 512, size: 0 },
{ bufferSize: 1024, offset: 512, size: 512 },
{ bufferSize: 1024, offset: 512, size: 512 + kSizeAlignment }]
)
).
fn(async (t) => {
  const { mapMode, bufferSize, size, offset } = t.params;
  const buffer = t.createMappableBuffer(mapMode, bufferSize);

  const successParam =
  offset + size <= bufferSize ?
  'success' :
  {
    validationError: true,
    earlyRejection: false,
    rejectName: 'OperationError'
  };
  await t.testMapAsyncCall(successParam, buffer, mapMode, offset, size);
});

g.test('mapAsync,earlyRejection').
desc("Test that mapAsync fails immediately if it's pending map.").
paramsSubcasesOnly((u) => u.combine('mapMode', kMapModeOptions).combine('offset2', [0, 8])).
fn(async (t) => {
  const { mapMode, offset2 } = t.params;

  const bufferSize = 16;
  const mapSize = 8;
  const offset1 = 0;

  const buffer = t.createMappableBuffer(mapMode, bufferSize);
  const p1 = buffer.mapAsync(mapMode, offset1, mapSize); // succeeds
  await t.testMapAsyncCall(
    {
      validationError: false,
      earlyRejection: true,
      rejectName: 'OperationError'
    },
    buffer,
    mapMode,
    offset2,
    mapSize
  );
  await p1; // ensure the original map still succeeds
});

g.test('mapAsync,abort_over_invalid_error').
desc(
  `Test that unmap abort error should have precedence over validation error
TODO
  - Add other validation error test (eg. offset is not a multiple of 8)
  `
).
paramsSubcasesOnly((u) =>
u.combine('mapMode', kMapModeOptions).combine('unmapBeforeResolve', [true, false])
).
fn(async (t) => {
  const { mapMode, unmapBeforeResolve } = t.params;
  const bufferSize = 8;
  const buffer = t.createMappableBuffer(mapMode, bufferSize);
  await buffer.mapAsync(mapMode);

  if (unmapBeforeResolve) {
    // unmap abort error should have precedence over validation error
    const pending = t.testMapAsyncCall(
      { validationError: true, earlyRejection: false, rejectName: 'AbortError' },
      buffer,
      mapMode
    );
    buffer.unmap();
    await pending;
  } else {
    // map on already mapped buffer should cause validation error
    await t.testMapAsyncCall(
      { validationError: true, earlyRejection: false, rejectName: 'OperationError' },
      buffer,
      mapMode
    );
    buffer.unmap();
  }
});

g.test('getMappedRange,state,mapped').
desc('Test that it is valid to call getMappedRange in the mapped state').
paramsSubcasesOnly((u) => u.combine('mapMode', kMapModeOptions)).
fn(async (t) => {
  const { mapMode } = t.params;
  const bufferSize = 16;
  const buffer = t.createMappableBuffer(mapMode, bufferSize);
  await buffer.mapAsync(mapMode);

  const data = buffer.getMappedRange();
  t.expect(data instanceof ArrayBuffer);
  t.expect(data.byteLength === bufferSize);

  // map on already mapped buffer should be rejected
  const pending = t.testMapAsyncCall(
    { validationError: true, earlyRejection: false, rejectName: 'OperationError' },
    buffer,
    mapMode
  );
  t.expect(data.byteLength === bufferSize);
  await pending;

  buffer.unmap();

  t.expect(data.byteLength === 0);
});

g.test('getMappedRange,state,mappedAtCreation').
desc(
  `Test that, in the mapped-at-creation state, it is valid to call getMappedRange, for all buffer usages,
    and invalid to call mapAsync, for all map modes.`
).
paramsSubcasesOnly((u) =>
u.combine('bufferUsage', kBufferUsages).combine('mapMode', kMapModeOptions)
).
fn(async (t) => {
  const { bufferUsage, mapMode } = t.params;
  const bufferSize = 16;
  const buffer = t.device.createBuffer({
    usage: bufferUsage,
    size: bufferSize,
    mappedAtCreation: true
  });

  const data = buffer.getMappedRange();
  t.expect(data instanceof ArrayBuffer);
  t.expect(data.byteLength === bufferSize);

  // map on already mapped buffer should be rejected
  const pending = t.testMapAsyncCall(
    { validationError: true, earlyRejection: false, rejectName: 'OperationError' },
    buffer,
    mapMode
  );
  t.expect(data.byteLength === bufferSize);
  await pending;

  buffer.unmap();

  t.expect(data.byteLength === 0);
});

g.test('getMappedRange,state,invalid_mappedAtCreation').
desc(
  `mappedAtCreation should return a mapped buffer, even if the buffer is invalid.
Like VRAM allocation (see map_oom), validation can be performed asynchronously (in the GPU process)
so the Content process doesn't necessarily know the buffer is invalid.`
).
fn((t) => {
  const buffer = t.expectGPUError('validation', () =>
  t.device.createBuffer({
    mappedAtCreation: true,
    size: 16,
    usage: 0xffff_ffff // Invalid usage
  })
  );

  // Should still be valid.
  buffer.getMappedRange();
});

g.test('getMappedRange,state,mappedAgain').
desc(
  'Test that it is valid to call getMappedRange in the mapped state, even if there is a duplicate mapAsync before'
).
paramsSubcasesOnly((u) => u.combine('mapMode', kMapModeOptions)).
fn(async (t) => {
  const { mapMode } = t.params;
  const buffer = t.createMappableBuffer(mapMode, 16);
  await buffer.mapAsync(mapMode);

  // call mapAsync again on already mapped buffer should fail
  await t.testMapAsyncCall(
    { validationError: true, earlyRejection: false, rejectName: 'OperationError' },
    buffer,
    mapMode
  );

  // getMapppedRange should still success
  t.testGetMappedRangeCall(true, buffer);
});

g.test('getMappedRange,state,unmapped').
desc(
  `Test that it is invalid to call getMappedRange in the unmapped state.
Test for various cases of being unmapped: at creation, after a mapAsync call or after being created mapped.`
).
fn(async (t) => {
  // It is invalid to call getMappedRange when the buffer starts unmapped when created.
  {
    const buffer = t.createMappableBuffer(GPUMapMode.READ, 16);
    t.testGetMappedRangeCall(false, buffer);
  }

  // It is invalid to call getMappedRange when the buffer is unmapped after mapAsync.
  {
    const buffer = t.createMappableBuffer(GPUMapMode.READ, 16);
    await buffer.mapAsync(GPUMapMode.READ);
    buffer.unmap();
    t.testGetMappedRangeCall(false, buffer);
  }

  // It is invalid to call getMappedRange when the buffer is unmapped after mappedAtCreation.
  {
    const buffer = t.device.createBuffer({
      usage: GPUBufferUsage.MAP_READ,
      size: 16,
      mappedAtCreation: true
    });
    buffer.unmap();
    t.testGetMappedRangeCall(false, buffer);
  }
});

g.test('getMappedRange,subrange,mapped').
desc(
  `Test that old getMappedRange returned arraybuffer does not exist after unmap, and newly returned
    arraybuffer after new map has correct subrange`
).
params((u) => u.combine('mapMode', kMapModeOptions)).
fn(async (t) => {
  const { mapMode } = t.params;
  const bufferSize = 16;
  const offset = 8;
  const subrangeSize = bufferSize - offset;
  const buffer = t.createMappableBuffer(mapMode, bufferSize);
  await buffer.mapAsync(mapMode);

  const data0 = buffer.getMappedRange();
  t.expect(data0 instanceof ArrayBuffer);
  t.expect(data0.byteLength === bufferSize);

  buffer.unmap();
  t.expect(data0.byteLength === 0);

  await buffer.mapAsync(mapMode, offset);
  const data1 = buffer.getMappedRange(8);

  t.expect(data0.byteLength === 0);
  t.expect(data1.byteLength === subrangeSize);
});

g.test('getMappedRange,subrange,mappedAtCreation').
desc(
  `Test that old getMappedRange returned arraybuffer does not exist after unmap and newly returned
    arraybuffer after new map has correct subrange`
).
fn(async (t) => {
  const bufferSize = 16;
  const offset = 8;
  const subrangeSize = bufferSize - offset;
  const buffer = t.device.createBuffer({
    size: bufferSize,
    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
    mappedAtCreation: true
  });

  const data0 = buffer.getMappedRange();
  t.expect(data0 instanceof ArrayBuffer);
  t.expect(data0.byteLength === bufferSize);

  buffer.unmap();
  t.expect(data0.byteLength === 0);

  await buffer.mapAsync(GPUMapMode.READ, offset);
  const data1 = buffer.getMappedRange(8);

  t.expect(data0.byteLength === 0);
  t.expect(data1.byteLength === subrangeSize);
});

g.test('getMappedRange,state,destroyed').
desc(
  `Test that it is invalid to call getMappedRange in the destroyed state.
Test for various cases of being destroyed: at creation, after a mapAsync call or after being created mapped.`
).
fn(async (t) => {
  // It is invalid to call getMappedRange when the buffer is destroyed when unmapped.
  {
    const buffer = t.createMappableBuffer(GPUMapMode.READ, 16);
    buffer.destroy();
    t.testGetMappedRangeCall(false, buffer);
  }

  // It is invalid to call getMappedRange when the buffer is destroyed when mapped.
  {
    const buffer = t.createMappableBuffer(GPUMapMode.READ, 16);
    await buffer.mapAsync(GPUMapMode.READ);
    buffer.destroy();
    t.testGetMappedRangeCall(false, buffer);
  }

  // It is invalid to call getMappedRange when the buffer is destroyed when mapped at creation.
  {
    const buffer = t.device.createBuffer({
      usage: GPUBufferUsage.MAP_READ,
      size: 16,
      mappedAtCreation: true
    });
    buffer.destroy();
    t.testGetMappedRangeCall(false, buffer);
  }
});

g.test('getMappedRange,state,mappingPending').
desc(`Test that it is invalid to call getMappedRange in the mappingPending state.`).
paramsSubcasesOnly((u) => u.combine('mapMode', kMapModeOptions)).
fn(async (t) => {
  const { mapMode } = t.params;
  const buffer = t.createMappableBuffer(mapMode, 16);

  /* noawait */const mapping0 = buffer.mapAsync(mapMode);
  // seconding mapping should be rejected
  const mapping1 = t.testMapAsyncCall(
    { validationError: false, earlyRejection: true, rejectName: 'OperationError' },
    buffer,
    mapMode
  );

  // invalid in mappingPending state
  t.testGetMappedRangeCall(false, buffer);

  await mapping0;

  // valid after buffer is mapped
  t.testGetMappedRangeCall(true, buffer);

  await mapping1;
});

g.test('getMappedRange,offsetAndSizeAlignment,mapped').
desc(`Test that getMappedRange fails if the alignment of offset and size isn't correct.`).
params((u) =>
u.
combine('mapMode', kMapModeOptions).
beginSubcases().
combine('mapOffset', [0, kOffsetAlignment]).
combine('offset', [0, kOffsetAlignment, kOffsetAlignment / 2]).
combine('size', [0, kSizeAlignment, kSizeAlignment / 2])
).
fn(async (t) => {
  const { mapMode, mapOffset, offset, size } = t.params;
  const buffer = t.createMappableBuffer(mapMode, 32);
  await buffer.mapAsync(mapMode, mapOffset);

  const success = offset % kOffsetAlignment === 0 && size % kSizeAlignment === 0;
  t.testGetMappedRangeCall(success, buffer, offset + mapOffset, size);
});

g.test('getMappedRange,offsetAndSizeAlignment,mappedAtCreation').
desc(`Test that getMappedRange fails if the alignment of offset and size isn't correct.`).
paramsSubcasesOnly((u) =>
u.
combine('offset', [0, kOffsetAlignment, kOffsetAlignment / 2]).
combine('size', [0, kSizeAlignment, kSizeAlignment / 2])
).
fn((t) => {
  const { offset, size } = t.params;
  const buffer = t.device.createBuffer({
    size: 16,
    usage: GPUBufferUsage.COPY_DST,
    mappedAtCreation: true
  });
  const success = offset % kOffsetAlignment === 0 && size % kSizeAlignment === 0;
  t.testGetMappedRangeCall(success, buffer, offset, size);
});

g.test('getMappedRange,sizeAndOffsetOOB,mappedAtCreation').
desc(
  `Test that getMappedRange size + offset must be less than the buffer size for a
    buffer mapped at creation. (and offset has not constraints on its own)`
).
paramsSubcasesOnly([
// Tests for a zero-sized buffer, with and without a size defined.
{ bufferSize: 0, offset: undefined, size: undefined },
{ bufferSize: 0, offset: undefined, size: 0 },
{ bufferSize: 0, offset: undefined, size: kSizeAlignment },
{ bufferSize: 0, offset: 0, size: undefined },
{ bufferSize: 0, offset: 0, size: 0 },
{ bufferSize: 0, offset: kOffsetAlignment, size: undefined },
{ bufferSize: 0, offset: kOffsetAlignment, size: 0 },

// Tests for a non-empty buffer, with an undefined offset.
{ bufferSize: 80, offset: undefined, size: 80 },
{ bufferSize: 80, offset: undefined, size: 80 + kSizeAlignment },

// Tests for a non-empty buffer, with an undefined size.
{ bufferSize: 80, offset: undefined, size: undefined },
{ bufferSize: 80, offset: 0, size: undefined },
{ bufferSize: 80, offset: kOffsetAlignment, size: undefined },
{ bufferSize: 80, offset: 80, size: undefined },
{ bufferSize: 80, offset: 80 + kOffsetAlignment, size: undefined },

// Tests for a non-empty buffer with a size defined.
{ bufferSize: 80, offset: 0, size: 80 },
{ bufferSize: 80, offset: 0, size: 80 + kSizeAlignment },
{ bufferSize: 80, offset: kOffsetAlignment, size: 80 },

{ bufferSize: 80, offset: 40, size: 40 },
{ bufferSize: 80, offset: 40 + kOffsetAlignment, size: 40 },
{ bufferSize: 80, offset: 40, size: 40 + kSizeAlignment }]
).
fn((t) => {
  const { bufferSize, offset, size } = t.params;
  const buffer = t.device.createBuffer({
    size: bufferSize,
    usage: GPUBufferUsage.COPY_DST,
    mappedAtCreation: true
  });

  const actualOffset = offset ?? 0;
  const actualSize = size ?? bufferSize - actualOffset;

  const success = actualOffset <= bufferSize && actualOffset + actualSize <= bufferSize;
  t.testGetMappedRangeCall(success, buffer, offset, size);
});

g.test('getMappedRange,sizeAndOffsetOOB,mapped').
desc('Test that getMappedRange size + offset must be less than the mapAsync range.').
paramsSubcasesOnly((u) =>
u //
.combine('mapMode', kMapModeOptions).
combineWithParams([
// Tests for an empty buffer, and implicit mapAsync size.
{ bufferSize: 0, mapOffset: 0, mapSize: undefined, offset: undefined, size: undefined },
{ bufferSize: 0, mapOffset: 0, mapSize: undefined, offset: undefined, size: 0 },
{
  bufferSize: 0,
  mapOffset: 0,
  mapSize: undefined,
  offset: undefined,
  size: kSizeAlignment
},
{ bufferSize: 0, mapOffset: 0, mapSize: undefined, offset: 0, size: undefined },
{ bufferSize: 0, mapOffset: 0, mapSize: undefined, offset: 0, size: 0 },
{
  bufferSize: 0,
  mapOffset: 0,
  mapSize: undefined,
  offset: kOffsetAlignment,
  size: undefined
},
{ bufferSize: 0, mapOffset: 0, mapSize: undefined, offset: kOffsetAlignment, size: 0 },

// Tests for an empty buffer, and explicit mapAsync size.
{ bufferSize: 0, mapOffset: 0, mapSize: 0, offset: undefined, size: undefined },
{ bufferSize: 0, mapOffset: 0, mapSize: 0, offset: 0, size: undefined },
{ bufferSize: 0, mapOffset: 0, mapSize: 0, offset: 0, size: 0 },
{ bufferSize: 0, mapOffset: 0, mapSize: 0, offset: kOffsetAlignment, size: undefined },
{ bufferSize: 0, mapOffset: 0, mapSize: 0, offset: kOffsetAlignment, size: 0 },

// Test for a fully implicit mapAsync call
{ bufferSize: 80, mapOffset: undefined, mapSize: undefined, offset: 0, size: 80 },
{
  bufferSize: 80,
  mapOffset: undefined,
  mapSize: undefined,
  offset: 0,
  size: 80 + kSizeAlignment
},
{
  bufferSize: 80,
  mapOffset: undefined,
  mapSize: undefined,
  offset: kOffsetAlignment,
  size: 80
},

// Test for a mapAsync call with an implicit size
{ bufferSize: 80, mapOffset: 24, mapSize: undefined, offset: 24, size: 80 - 24 },
{
  bufferSize: 80,
  mapOffset: 24,
  mapSize: undefined,
  offset: 0,
  size: 80 - 24 + kSizeAlignment
},
{
  bufferSize: 80,
  mapOffset: 24,
  mapSize: undefined,
  offset: kOffsetAlignment,
  size: 80 - 24
},

// Test for a non-empty buffer fully mapped.
{ bufferSize: 80, mapOffset: 0, mapSize: 80, offset: 0, size: 80 },
{ bufferSize: 80, mapOffset: 0, mapSize: 80, offset: kOffsetAlignment, size: 80 },
{ bufferSize: 80, mapOffset: 0, mapSize: 80, offset: 0, size: 80 + kSizeAlignment },

{ bufferSize: 80, mapOffset: 0, mapSize: 80, offset: 40, size: 40 },
{ bufferSize: 80, mapOffset: 0, mapSize: 80, offset: 40 + kOffsetAlignment, size: 40 },
{ bufferSize: 80, mapOffset: 0, mapSize: 80, offset: 40, size: 40 + kSizeAlignment },

// Test for a buffer partially mapped.
{ bufferSize: 80, mapOffset: 24, mapSize: 40, offset: 24, size: 40 },
{ bufferSize: 80, mapOffset: 24, mapSize: 40, offset: 24 - kOffsetAlignment, size: 40 },
{ bufferSize: 80, mapOffset: 24, mapSize: 40, offset: 24 + kOffsetAlignment, size: 40 },
{ bufferSize: 80, mapOffset: 24, mapSize: 40, offset: 24, size: 40 + kSizeAlignment },

// Test for a partially mapped buffer with implicit size and offset for getMappedRange.
// - Buffer partially mapped in the middle
{ bufferSize: 80, mapOffset: 24, mapSize: 40, offset: undefined, size: undefined },
{ bufferSize: 80, mapOffset: 24, mapSize: 40, offset: 0, size: undefined },
{ bufferSize: 80, mapOffset: 24, mapSize: 40, offset: 24, size: undefined },
// - Buffer partially mapped to the end
{ bufferSize: 80, mapOffset: 24, mapSize: undefined, offset: 24, size: undefined },
{ bufferSize: 80, mapOffset: 24, mapSize: undefined, offset: 80, size: undefined },
// - Buffer partially mapped from the start
{ bufferSize: 80, mapOffset: 0, mapSize: 64, offset: undefined, size: undefined },
{ bufferSize: 80, mapOffset: 0, mapSize: 64, offset: undefined, size: 64 }]
)
).
fn(async (t) => {
  const { mapMode, bufferSize, mapOffset, mapSize, offset, size } = t.params;
  const buffer = t.createMappableBuffer(mapMode, bufferSize);
  await buffer.mapAsync(mapMode, mapOffset, mapSize);

  const actualMapOffset = mapOffset ?? 0;
  const actualMapSize = mapSize ?? bufferSize - actualMapOffset;

  const actualOffset = offset ?? 0;
  const actualSize = size ?? bufferSize - actualOffset;

  const success =
  actualOffset >= actualMapOffset &&
  actualOffset <= bufferSize &&
  actualOffset + actualSize <= actualMapOffset + actualMapSize;
  t.testGetMappedRangeCall(success, buffer, offset, size);
});

g.test('getMappedRange,disjointRanges').
desc('Test that the ranges asked through getMappedRange must be disjoint.').
paramsSubcasesOnly((u) =>
u //
.combine('remapBetweenCalls', [false, true]).
combineWithParams([
// Disjoint ranges with one that's empty.
{ offset1: 8, size1: 0, offset2: 8, size2: 8 },
{ offset1: 16, size1: 0, offset2: 8, size2: 8 },

{ offset1: 8, size1: 8, offset2: 8, size2: 0 },
{ offset1: 8, size1: 8, offset2: 16, size2: 0 },

// Disjoint ranges with both non-empty.
{ offset1: 0, size1: 8, offset2: 8, size2: 8 },
{ offset1: 16, size1: 8, offset2: 8, size2: 8 },

{ offset1: 8, size1: 8, offset2: 0, size2: 8 },
{ offset1: 8, size1: 8, offset2: 16, size2: 8 },

// Empty range contained inside another one.
{ offset1: 16, size1: 20, offset2: 24, size2: 0 },
{ offset1: 24, size1: 0, offset2: 16, size2: 20 },

// Ranges that overlap only partially.
{ offset1: 16, size1: 20, offset2: 8, size2: 20 },
{ offset1: 16, size1: 20, offset2: 32, size2: 20 },

// Ranges that include one another.
{ offset1: 0, size1: 80, offset2: 16, size2: 20 },
{ offset1: 16, size1: 20, offset2: 0, size2: 80 }]
)
).
fn(async (t) => {
  const { offset1, size1, offset2, size2, remapBetweenCalls } = t.params;
  const buffer = t.device.createBuffer({ size: 80, usage: GPUBufferUsage.MAP_READ });
  await buffer.mapAsync(GPUMapMode.READ);

  t.testGetMappedRangeCall(true, buffer, offset1, size1);

  if (remapBetweenCalls) {
    buffer.unmap();
    await buffer.mapAsync(GPUMapMode.READ);
  }

  const range1StartsAfter2 = offset1 >= offset2 + size2;
  const range2StartsAfter1 = offset2 >= offset1 + size1;
  const disjoint = range1StartsAfter2 || range2StartsAfter1;
  const success = disjoint || remapBetweenCalls;

  t.testGetMappedRangeCall(success, buffer, offset2, size2);
});

g.test('getMappedRange,disjoinRanges_many').
desc('Test getting a lot of small ranges, and that the disjoint check checks them all.').
fn(async (t) => {
  const kStride = 256;
  const kNumStrides = 256;

  const buffer = t.device.createBuffer({
    size: kStride * kNumStrides,
    usage: GPUBufferUsage.MAP_READ
  });
  await buffer.mapAsync(GPUMapMode.READ);

  // Get a lot of small mapped ranges.
  for (let stride = 0; stride < kNumStrides; stride++) {
    t.testGetMappedRangeCall(true, buffer, stride * kStride, 8);
  }

  // Check for each range it is invalid to get a range that overlaps it and check that it is valid
  // to get ranges for the rest of the buffer.
  for (let stride = 0; stride < kNumStrides; stride++) {
    t.testGetMappedRangeCall(false, buffer, stride * kStride, kStride);
    t.testGetMappedRangeCall(true, buffer, stride * kStride + 8, kStride - 8);
  }
});

g.test('unmap,state,unmapped').
desc(
  `Test it is valid to call unmap on a buffer that is unmapped (at creation, or after
    mappedAtCreation or mapAsync)`
).
fn(async (t) => {
  // It is valid to call unmap after creation of an unmapped buffer.
  {
    const buffer = t.device.createBuffer({ size: 16, usage: GPUBufferUsage.MAP_READ });
    buffer.unmap();
  }

  // It is valid to call unmap after unmapping a mapAsynced buffer.
  {
    const buffer = t.createMappableBuffer(GPUMapMode.READ, 16);
    await buffer.mapAsync(GPUMapMode.READ);
    buffer.unmap();
    buffer.unmap();
  }

  // It is valid to call unmap after unmapping a mappedAtCreation buffer.
  {
    const buffer = t.device.createBuffer({
      usage: GPUBufferUsage.MAP_READ,
      size: 16,
      mappedAtCreation: true
    });
    buffer.unmap();
    buffer.unmap();
  }
});

g.test('unmap,state,destroyed').
desc(
  `Test it is valid to call unmap on a buffer that is destroyed (at creation, or after
    mappedAtCreation or mapAsync)`
).
fn(async (t) => {
  // It is valid to call unmap after destruction of an unmapped buffer.
  {
    const buffer = t.device.createBuffer({ size: 16, usage: GPUBufferUsage.MAP_READ });
    buffer.destroy();
    buffer.unmap();
  }

  // It is valid to call unmap after destroying a mapAsynced buffer.
  {
    const buffer = t.createMappableBuffer(GPUMapMode.READ, 16);
    await buffer.mapAsync(GPUMapMode.READ);
    buffer.destroy();
    buffer.unmap();
  }

  // It is valid to call unmap after destroying a mappedAtCreation buffer.
  {
    const buffer = t.device.createBuffer({
      usage: GPUBufferUsage.MAP_READ,
      size: 16,
      mappedAtCreation: true
    });
    buffer.destroy();
    buffer.unmap();
  }
});

g.test('unmap,state,mappedAtCreation').
desc('Test it is valid to call unmap on a buffer mapped at creation, for various usages').
paramsSubcasesOnly((u) =>
u //
.combine('bufferUsage', kBufferUsages)
).
fn((t) => {
  const { bufferUsage } = t.params;
  const buffer = t.device.createBuffer({ size: 16, usage: bufferUsage, mappedAtCreation: true });

  buffer.unmap();
});

g.test('unmap,state,mapped').
desc("Test it is valid to call unmap on a buffer that's mapped").
paramsSubcasesOnly((u) => u.combine('mapMode', kMapModeOptions)).
fn(async (t) => {
  const { mapMode } = t.params;
  const buffer = t.createMappableBuffer(mapMode, 16);

  await buffer.mapAsync(mapMode);
  buffer.unmap();
});

g.test('unmap,state,mappingPending').
desc("Test it is valid to call unmap on a buffer that's being mapped").
paramsSubcasesOnly((u) => u.combine('mapMode', kMapModeOptions)).
fn(async (t) => {
  const { mapMode } = t.params;
  const buffer = t.createMappableBuffer(mapMode, 16);

  const pending = t.testMapAsyncCall(
    { validationError: false, earlyRejection: false, rejectName: 'AbortError' },
    buffer,
    mapMode
  );
  buffer.unmap();
  await pending;
});

g.test('gc_behavior,mappedAtCreation').
desc(
  "Test that GCing the buffer while mappings are handed out doesn't invalidate them - mappedAtCreation case"
).
fn(async (t) => {
  let buffer = null;
  buffer = t.device.createBuffer({
    size: 256,
    usage: GPUBufferUsage.COPY_DST,
    mappedAtCreation: true
  });

  // Write some non-zero data to the buffer.
  const contents = new Uint32Array(buffer.getMappedRange());
  for (let i = 0; i < contents.length; i++) {
    contents[i] = i;
  }

  // Trigger garbage collection that should collect the buffer (or as if it collected it)
  // NOTE: This won't fail unless the browser immediately starts reusing the memory, or gives it
  // back to the OS. One good option for browsers to check their logic is good is to zero-out the
  // memory on GPUBuffer (or internal gpu::Buffer-like object) destruction.
  buffer = null;
  await attemptGarbageCollection();

  // Use the mapping again both for read and write, it should work.
  for (let i = 0; i < contents.length; i++) {
    t.expect(contents[i] === i);
    contents[i] = i + 1;
  }
});

g.test('gc_behavior,mapAsync').
desc(
  "Test that GCing the buffer while mappings are handed out doesn't invalidate them - mapAsync case"
).
paramsSubcasesOnly((u) => u.combine('mapMode', kMapModeOptions)).
fn(async (t) => {
  const { mapMode } = t.params;

  let buffer = null;
  buffer = t.createMappableBuffer(mapMode, 256);
  await buffer.mapAsync(mapMode);

  // Write some non-zero data to the buffer.
  const contents = new Uint32Array(buffer.getMappedRange());
  for (let i = 0; i < contents.length; i++) {
    contents[i] = i;
  }

  // Trigger garbage collection that should collect the buffer (or as if it collected it)
  // NOTE: This won't fail unless the browser immediately starts reusing the memory, or gives it
  // back to the OS. One good option for browsers to check their logic is good is to zero-out the
  // memory on GPUBuffer (or internal gpu::Buffer-like object) destruction.
  buffer = null;
  await attemptGarbageCollection();

  // Use the mapping again both for read and write, it should work.
  for (let i = 0; i < contents.length; i++) {
    t.expect(contents[i] === i);
    contents[i] = i + 1;
  }
});