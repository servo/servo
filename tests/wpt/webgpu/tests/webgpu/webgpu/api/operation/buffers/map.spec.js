/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test the operation of buffer mapping, specifically the data contents written via
map-write/mappedAtCreation, and the contents of buffers returned by getMappedRange on
buffers which are mapped-read/mapped-write/mappedAtCreation.

range: used for getMappedRange
mapRegion: used for mapAsync

mapRegionBoundModes is used to get mapRegion from range:
 - default-expand: expand mapRegion to buffer bound by setting offset/size to undefined
 - explicit-expand: expand mapRegion to buffer bound by explicitly calculating offset/size
 - minimal: make mapRegion to be the same as range which is the minimal range to make getMappedRange input valid
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert, memcpy } from '../../../../common/util/util.js';
import { checkElementsEqual } from '../../../util/check_contents.js';

import { MappingTest } from './mapping_test.js';

export const g = makeTestGroup(MappingTest);

const kSubcases = [
{ size: 0, range: [] },
{ size: 0, range: [undefined] },
{ size: 0, range: [undefined, undefined] },
{ size: 0, range: [0] },
{ size: 0, range: [0, undefined] },
{ size: 0, range: [0, 0] },
{ size: 12, range: [] },
{ size: 12, range: [undefined] },
{ size: 12, range: [undefined, undefined] },
{ size: 12, range: [0] },
{ size: 12, range: [0, undefined] },
{ size: 12, range: [0, 12] },
{ size: 12, range: [0, 0] },
{ size: 12, range: [8] },
{ size: 12, range: [8, undefined] },
{ size: 12, range: [8, 4] },
{ size: 28, range: [8, 8] },
{ size: 28, range: [8, 12] },
{ size: 512 * 1024, range: [] }];


function reifyMapRange(bufferSize, range) {
  const offset = range[0] ?? 0;
  return [offset, range[1] ?? bufferSize - offset];
}

const mapRegionBoundModes = ['default-expand', 'explicit-expand', 'minimal'];


function getRegionForMap(
bufferSize,
range,
{
  mapAsyncRegionLeft,
  mapAsyncRegionRight



})
{
  const regionLeft = mapAsyncRegionLeft === 'minimal' ? range[0] : 0;
  const regionRight = mapAsyncRegionRight === 'minimal' ? range[0] + range[1] : bufferSize;
  return [
  mapAsyncRegionLeft === 'default-expand' ? undefined : regionLeft,
  mapAsyncRegionRight === 'default-expand' ? undefined : regionRight - regionLeft];

}

g.test('mapAsync,write').
desc(
  `Use map-write to write to various ranges of variously-sized buffers, then expectContents
(which does copyBufferToBuffer + map-read) to ensure the contents were written.`
).
params((u) =>
u.
combine('mapAsyncRegionLeft', mapRegionBoundModes).
combine('mapAsyncRegionRight', mapRegionBoundModes).
beginSubcases().
combineWithParams(kSubcases)
).
fn(async (t) => {
  const { size, range } = t.params;
  const [rangeOffset, rangeSize] = reifyMapRange(size, range);

  const buffer = t.device.createBuffer({
    size,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.MAP_WRITE
  });

  const mapRegion = getRegionForMap(size, [rangeOffset, rangeSize], t.params);
  await buffer.mapAsync(GPUMapMode.WRITE, ...mapRegion);
  const arrayBuffer = buffer.getMappedRange(...range);
  t.checkMapWrite(buffer, rangeOffset, arrayBuffer, rangeSize);
});

g.test('mapAsync,write,unchanged_ranges_preserved').
desc(
  `Use mappedAtCreation or mapAsync to write to various ranges of variously-sized buffers, then
use mapAsync to map a different range and zero it out. Finally use expectGPUBufferValuesEqual
(which does copyBufferToBuffer + map-read) to verify that contents originally written outside the
second mapped range were not altered.`
).
params((u) =>
u.
beginSubcases().
combine('mappedAtCreation', [false, true]).
combineWithParams([
{ size: 12, range1: [], range2: [8] },
{ size: 12, range1: [], range2: [0, 8] },
{ size: 12, range1: [0, 8], range2: [8] },
{ size: 12, range1: [8], range2: [0, 8] },
{ size: 28, range1: [], range2: [8, 8] },
{ size: 28, range1: [8, 16], range2: [16, 8] },
{ size: 32, range1: [16, 12], range2: [8, 16] },
{ size: 32, range1: [8, 8], range2: [24, 4] }]
)
).
fn(async (t) => {
  const { size, range1, range2, mappedAtCreation } = t.params;
  const [rangeOffset1, rangeSize1] = reifyMapRange(size, range1);
  const [rangeOffset2, rangeSize2] = reifyMapRange(size, range2);

  const buffer = t.device.createBuffer({
    mappedAtCreation,
    size,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.MAP_WRITE
  });

  // If the buffer is not mappedAtCreation map it now.
  if (!mappedAtCreation) {
    await buffer.mapAsync(GPUMapMode.WRITE);
  }

  // Set the initial contents of the buffer.
  const init = buffer.getMappedRange(...range1);

  assert(init.byteLength === rangeSize1);
  const expectedBuffer = new ArrayBuffer(size);
  const expected = new Uint32Array(
    expectedBuffer,
    rangeOffset1,
    rangeSize1 / Uint32Array.BYTES_PER_ELEMENT
  );
  const data = new Uint32Array(init);
  for (let i = 0; i < data.length; ++i) {
    data[i] = expected[i] = i + 1;
  }
  buffer.unmap();

  // Write to a second range of the buffer
  await buffer.mapAsync(GPUMapMode.WRITE, ...range2);
  const init2 = buffer.getMappedRange(...range2);

  assert(init2.byteLength === rangeSize2);
  const expected2 = new Uint32Array(
    expectedBuffer,
    rangeOffset2,
    rangeSize2 / Uint32Array.BYTES_PER_ELEMENT
  );
  const data2 = new Uint32Array(init2);
  for (let i = 0; i < data2.length; ++i) {
    data2[i] = expected2[i] = 0;
  }
  buffer.unmap();

  // Verify that the range of the buffer which was not overwritten was preserved.
  t.expectGPUBufferValuesEqual(buffer, expected, rangeOffset1);
});

g.test('mapAsync,read').
desc(
  `Use mappedAtCreation to initialize various ranges of variously-sized buffers, then
map-read and check the read-back result.`
).
params((u) =>
u.
combine('mapAsyncRegionLeft', mapRegionBoundModes).
combine('mapAsyncRegionRight', mapRegionBoundModes).
beginSubcases().
combineWithParams(kSubcases)
).
fn(async (t) => {
  const { size, range } = t.params;
  const [rangeOffset, rangeSize] = reifyMapRange(size, range);

  const buffer = t.device.createBuffer({
    mappedAtCreation: true,
    size,
    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ
  });
  const init = buffer.getMappedRange(...range);

  assert(init.byteLength === rangeSize);
  const expected = new Uint32Array(new ArrayBuffer(rangeSize));
  const data = new Uint32Array(init);
  for (let i = 0; i < data.length; ++i) {
    data[i] = expected[i] = i + 1;
  }
  buffer.unmap();

  const mapRegion = getRegionForMap(size, [rangeOffset, rangeSize], t.params);
  await buffer.mapAsync(GPUMapMode.READ, ...mapRegion);
  const actual = new Uint8Array(buffer.getMappedRange(...range));
  t.expectOK(checkElementsEqual(actual, new Uint8Array(expected.buffer)));
});

g.test('mapAsync,read,typedArrayAccess').
desc(`Use various TypedArray types to read back from a mapped buffer`).
params((u) =>
u.
combine('mapAsyncRegionLeft', mapRegionBoundModes).
combine('mapAsyncRegionRight', mapRegionBoundModes).
beginSubcases().
combineWithParams([
{ size: 80, range: [] },
{ size: 160, range: [] },
{ size: 160, range: [0, 80] },
{ size: 160, range: [80] },
{ size: 160, range: [40, 120] },
{ size: 160, range: [40] }]
)
).
fn(async (t) => {
  const { size, range } = t.params;
  const [rangeOffset, rangeSize] = reifyMapRange(size, range);

  // Fill an array buffer with a variety of values of different types.
  const expectedArrayBuffer = new ArrayBuffer(80);
  const uint8Expected = new Uint8Array(expectedArrayBuffer, 0, 2);
  uint8Expected[0] = 1;
  uint8Expected[1] = 255;

  const int8Expected = new Int8Array(expectedArrayBuffer, 2, 2);
  int8Expected[0] = -1;
  int8Expected[1] = 127;

  const uint16Expected = new Uint16Array(expectedArrayBuffer, 4, 2);
  uint16Expected[0] = 1;
  uint16Expected[1] = 65535;

  const int16Expected = new Int16Array(expectedArrayBuffer, 8, 2);
  int16Expected[0] = -1;
  int16Expected[1] = 32767;

  const uint32Expected = new Uint32Array(expectedArrayBuffer, 12, 2);
  uint32Expected[0] = 1;
  uint32Expected[1] = 4294967295;

  const int32Expected = new Int32Array(expectedArrayBuffer, 20, 2);
  int32Expected[2] = -1;
  int32Expected[3] = 2147483647;

  const float32Expected = new Float32Array(expectedArrayBuffer, 28, 3);
  float32Expected[0] = 1;
  float32Expected[1] = -1;
  float32Expected[2] = 12345.6789;

  const float64Expected = new Float64Array(expectedArrayBuffer, 40, 5);
  float64Expected[0] = 1;
  float64Expected[1] = -1;
  float64Expected[2] = 12345.6789;
  float64Expected[3] = Number.MAX_VALUE;
  float64Expected[4] = Number.MIN_VALUE;

  const buffer = t.device.createBuffer({
    mappedAtCreation: true,
    size,
    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ
  });
  const init = buffer.getMappedRange(...range);

  // Copy the expected values into the mapped range.
  assert(init.byteLength === rangeSize);
  memcpy({ src: expectedArrayBuffer }, { dst: init });
  buffer.unmap();

  const mapRegion = getRegionForMap(size, [rangeOffset, rangeSize], t.params);
  await buffer.mapAsync(GPUMapMode.READ, ...mapRegion);
  const mappedArrayBuffer = buffer.getMappedRange(...range);
  t.expectOK(checkElementsEqual(new Uint8Array(mappedArrayBuffer, 0, 2), uint8Expected));
  t.expectOK(checkElementsEqual(new Int8Array(mappedArrayBuffer, 2, 2), int8Expected));
  t.expectOK(checkElementsEqual(new Uint16Array(mappedArrayBuffer, 4, 2), uint16Expected));
  t.expectOK(checkElementsEqual(new Int16Array(mappedArrayBuffer, 8, 2), int16Expected));
  t.expectOK(checkElementsEqual(new Uint32Array(mappedArrayBuffer, 12, 2), uint32Expected));
  t.expectOK(checkElementsEqual(new Int32Array(mappedArrayBuffer, 20, 2), int32Expected));
  t.expectOK(checkElementsEqual(new Float32Array(mappedArrayBuffer, 28, 3), float32Expected));
  t.expectOK(checkElementsEqual(new Float64Array(mappedArrayBuffer, 40, 5), float64Expected));
});

g.test('mappedAtCreation').
desc(
  `Use mappedAtCreation to write to various ranges of variously-sized buffers created either
with or without the MAP_WRITE usage (since this could affect the mappedAtCreation upload path),
then expectContents (which does copyBufferToBuffer + map-read) to ensure the contents were written.`
).
params((u) =>
u //
.combine('mappable', [false, true]).
beginSubcases().
combineWithParams(kSubcases)
).
fn((t) => {
  const { size, range, mappable } = t.params;
  const [, rangeSize] = reifyMapRange(size, range);

  const buffer = t.device.createBuffer({
    mappedAtCreation: true,
    size,
    usage: GPUBufferUsage.COPY_SRC | (mappable ? GPUBufferUsage.MAP_WRITE : 0)
  });
  const arrayBuffer = buffer.getMappedRange(...range);
  t.checkMapWrite(buffer, range[0] ?? 0, arrayBuffer, rangeSize);
});

g.test('remapped_for_write').
desc(
  `Use mappedAtCreation or mapAsync to write to various ranges of variously-sized buffers created
with the MAP_WRITE usage, then mapAsync again and ensure that the previously written values are
still present in the mapped buffer.`
).
params((u) =>
u //
.combine('mapAsyncRegionLeft', mapRegionBoundModes).
combine('mapAsyncRegionRight', mapRegionBoundModes).
beginSubcases().
combine('mappedAtCreation', [false, true]).
combineWithParams(kSubcases)
).
fn(async (t) => {
  const { size, range, mappedAtCreation } = t.params;
  const [rangeOffset, rangeSize] = reifyMapRange(size, range);

  const buffer = t.device.createBuffer({
    mappedAtCreation,
    size,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.MAP_WRITE
  });

  // If the buffer is not mappedAtCreation map it now.
  if (!mappedAtCreation) {
    await buffer.mapAsync(GPUMapMode.WRITE);
  }

  // Set the initial contents of the buffer.
  const init = buffer.getMappedRange(...range);

  assert(init.byteLength === rangeSize);
  const expected = new Uint32Array(new ArrayBuffer(rangeSize));
  const data = new Uint32Array(init);
  for (let i = 0; i < data.length; ++i) {
    data[i] = expected[i] = i + 1;
  }
  buffer.unmap();

  // Check that upon remapping the for WRITE the values in the buffer are
  // still the same.
  const mapRegion = getRegionForMap(size, [rangeOffset, rangeSize], t.params);
  await buffer.mapAsync(GPUMapMode.WRITE, ...mapRegion);
  const actual = new Uint8Array(buffer.getMappedRange(...range));
  t.expectOK(checkElementsEqual(actual, new Uint8Array(expected.buffer)));
});

g.test('mappedAtCreation,mapState').
desc('Test that exposed map state of buffer created with mappedAtCreation has expected values.').
params((u) =>
u.
combine('usageType', ['invalid', 'read', 'write']).
combine('afterUnmap', [false, true]).
combine('afterDestroy', [false, true])
).
fn((t) => {
  const { usageType, afterUnmap, afterDestroy } = t.params;
  const usage =
  usageType === 'read' ?
  GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ :
  usageType === 'write' ?
  GPUBufferUsage.COPY_SRC | GPUBufferUsage.MAP_WRITE :
  0;
  const validationError = usage === 0;
  const size = 8;
  const range = [0, 8];

  let buffer;
  t.expectValidationError(() => {
    buffer = t.device.createBuffer({
      mappedAtCreation: true,
      size,
      usage
    });
  }, validationError);

  // mapState must be "mapped" regardless of validation error
  t.expect(buffer.mapState === 'mapped');

  // getMappedRange must not change the map state
  buffer.getMappedRange(...range);
  t.expect(buffer.mapState === 'mapped');

  if (afterUnmap) {
    buffer.unmap();
    t.expect(buffer.mapState === 'unmapped');
  }

  if (afterDestroy) {
    buffer.destroy();
    t.expect(buffer.mapState === 'unmapped');
  }
});

g.test('mapAsync,mapState').
desc('Test that exposed map state of buffer mapped with mapAsync has expected values.').
params((u) =>
u.
combine('usageType', ['invalid', 'read', 'write']).
combine('mapModeType', ['READ', 'WRITE']).
combine('beforeUnmap', [false, true]).
combine('beforeDestroy', [false, true]).
combine('afterUnmap', [false, true]).
combine('afterDestroy', [false, true])
).
fn(async (t) => {
  const { usageType, mapModeType, beforeUnmap, beforeDestroy, afterUnmap, afterDestroy } =
  t.params;
  const size = 8;
  const range = [0, 8];
  const usage =
  usageType === 'read' ?
  GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ :
  usageType === 'write' ?
  GPUBufferUsage.COPY_SRC | GPUBufferUsage.MAP_WRITE :
  0;
  const bufferCreationValidationError = usage === 0;
  const mapMode = GPUMapMode[mapModeType];

  let buffer;
  t.expectValidationError(() => {
    buffer = t.device.createBuffer({
      mappedAtCreation: false,
      size,
      usage
    });
  }, bufferCreationValidationError);

  t.expect(buffer.mapState === 'unmapped');

  {
    const mapAsyncValidationError =
    bufferCreationValidationError ||
    mapMode === GPUMapMode.READ && !(usage & GPUBufferUsage.MAP_READ) ||
    mapMode === GPUMapMode.WRITE && !(usage & GPUBufferUsage.MAP_WRITE);
    let promise;
    t.expectValidationError(() => {
      promise = buffer.mapAsync(mapMode);
    }, mapAsyncValidationError);
    t.expect(buffer.mapState === 'pending');

    try {
      if (beforeUnmap) {
        buffer.unmap();
        t.expect(buffer.mapState === 'unmapped');
      }
      if (beforeDestroy) {
        buffer.destroy();
        t.expect(buffer.mapState === 'unmapped');
      }

      await promise;
      t.expect(buffer.mapState === 'mapped');

      // getMappedRange must not change the map state
      buffer.getMappedRange(...range);
      t.expect(buffer.mapState === 'mapped');
    } catch {
      // unmapped before resolve, destroyed before resolve, or mapAsync validation error
      // will end up with rejection and 'unmapped'
      t.expect(buffer.mapState === 'unmapped');
    }
  }

  // If buffer is already mapped test mapAsync on already mapped buffer
  if (buffer.mapState === 'mapped') {
    // mapAsync on already mapped buffer must be rejected with a validation error
    // and the map state must keep 'mapped'
    let promise;
    t.expectValidationError(() => {
      promise = buffer.mapAsync(GPUMapMode.WRITE);
    }, true);
    t.expect(buffer.mapState === 'mapped');

    try {
      await promise;
      t.fail('mapAsync on already mapped buffer must not succeed.');
    } catch {
      t.expect(buffer.mapState === 'mapped');
    }
  }

  if (afterUnmap) {
    buffer.unmap();
    t.expect(buffer.mapState === 'unmapped');
  }

  if (afterDestroy) {
    buffer.destroy();
    t.expect(buffer.mapState === 'unmapped');
  }
});