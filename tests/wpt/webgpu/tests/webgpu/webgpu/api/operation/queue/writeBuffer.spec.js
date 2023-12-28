/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = 'Operation tests for GPUQueue.writeBuffer()';import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { memcpy, range } from '../../../../common/util/util.js';
import { GPUTest } from '../../../gpu_test.js';
import { align } from '../../../util/math.js';

const kTypedArrays = [
'Uint8Array',
'Uint16Array',
'Uint32Array',
'Int8Array',
'Int16Array',
'Int32Array',
'Float32Array',
'Float64Array'];











class F extends GPUTest {
  calculateRequiredBufferSize(writes) {
    let bufferSize = 0;
    // Calculate size of final buffer
    for (const { bufferOffset, data, arrayType, useArrayBuffer, dataOffset, dataSize } of writes) {
      const TypedArrayConstructor = globalThis[arrayType];

      // When passing data as an ArrayBuffer, dataOffset and dataSize use byte instead of number of
      // elements. bytesPerElement is used to convert dataOffset and dataSize from elements to bytes
      // when useArrayBuffer === false.
      const bytesPerElement = useArrayBuffer ? 1 : TypedArrayConstructor.BYTES_PER_ELEMENT;

      // Calculate the number of bytes written to the buffer. data is always an array of elements.
      let bytesWritten =
      data.length * TypedArrayConstructor.BYTES_PER_ELEMENT - (dataOffset || 0) * bytesPerElement;

      if (dataSize) {
        // When defined, dataSize clamps the number of bytes written
        bytesWritten = Math.min(bytesWritten, dataSize * bytesPerElement);
      }

      // The minimum buffer size required for the write to succeed is the number of bytes written +
      // the bufferOffset
      const requiredBufferSize = bufferOffset + bytesWritten;

      // Find the largest required size by all writes
      bufferSize = Math.max(bufferSize, requiredBufferSize);
    }
    // writeBuffer requires buffers to be a multiple of 4
    return align(bufferSize, 4);
  }

  testWriteBuffer(...writes) {
    const bufferSize = this.calculateRequiredBufferSize(writes);

    // Initialize buffer to non-zero data (0xff) for easier debug.
    const expectedData = new Uint8Array(bufferSize).fill(0xff);

    const buffer = this.makeBufferWithContents(
      expectedData,
      GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
    );

    for (const { bufferOffset, data, arrayType, useArrayBuffer, dataOffset, dataSize } of writes) {
      const TypedArrayConstructor = globalThis[arrayType];
      const writeData = new TypedArrayConstructor(data);
      const writeSrc = useArrayBuffer ? writeData.buffer : writeData;
      this.queue.writeBuffer(buffer, bufferOffset, writeSrc, dataOffset, dataSize);
      memcpy(
        { src: writeSrc, start: dataOffset, length: dataSize },
        { dst: expectedData, start: bufferOffset }
      );
    }

    this.debug(`expectedData: [${expectedData.join(', ')}]`);
    this.expectGPUBufferValuesEqual(buffer, expectedData);
  }
}

export const g = makeTestGroup(F);

const kTestData = range(16, (i) => i);

g.test('array_types').
desc('Tests that writeBuffer correctly handles different TypedArrays and ArrayBuffer.').
params((u) =>
u //
.combine('arrayType', kTypedArrays).
combine('useArrayBuffer', [false, true])
).
fn((t) => {
  const { arrayType, useArrayBuffer } = t.params;
  const dataOffset = 1;
  const dataSize = 8;
  t.testWriteBuffer({
    bufferOffset: 0,
    arrayType,
    data: kTestData,
    dataOffset,
    dataSize,
    useArrayBuffer
  });
});

g.test('multiple_writes_at_different_offsets_and_sizes').
desc(
  `
Tests that writeBuffer currently handles different offsets and writes. This includes:
- Non-overlapping TypedArrays and ArrayLists
- Overlapping TypedArrays and ArrayLists
- Writing zero data
- Writing on zero sized buffers
- Unaligned source
- Multiple overlapping writes with decreasing sizes
    `
).
paramsSubcasesOnly([
{
  // Concatenate 2 Uint32Arrays
  writes: [
  {
    bufferOffset: 0,
    data: kTestData,
    arrayType: 'Uint32Array',
    useArrayBuffer: false,
    dataOffset: 2,
    dataSize: 2
  }, // [2, 3]
  {
    bufferOffset: 2 * Uint32Array.BYTES_PER_ELEMENT,
    data: kTestData,
    arrayType: 'Uint32Array',
    useArrayBuffer: false,
    dataOffset: 0,
    dataSize: 2
  } // [0, 1]
  ] // Expected [2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0]
},
{
  // Concatenate 2 Uint8Arrays
  writes: [
  { bufferOffset: 0, data: [0, 1, 2, 3], arrayType: 'Uint8Array', useArrayBuffer: false },
  { bufferOffset: 4, data: [4, 5, 6, 7], arrayType: 'Uint8Array', useArrayBuffer: false }]
  // Expected [0, 1, 2, 3, 4, 5, 6, 7]
},
{
  // Overlap in the middle
  writes: [
  { bufferOffset: 0, data: kTestData, arrayType: 'Uint8Array', useArrayBuffer: false },
  { bufferOffset: 4, data: [0], arrayType: 'Uint32Array', useArrayBuffer: false }]
  // Expected [0, 1, 2, 3, 0, 0 ,0 ,0, 8, 9, 10, 11, 12, 13, 14, 15]
},
{
  // Overlapping arrayLists
  writes: [
  {
    bufferOffset: 0,
    data: kTestData,
    arrayType: 'Uint32Array',
    useArrayBuffer: true,
    dataOffset: 2,
    dataSize: 4 * Uint32Array.BYTES_PER_ELEMENT
  },
  { bufferOffset: 4, data: [0x04030201], arrayType: 'Uint32Array', useArrayBuffer: true }]
  // Expected [0, 0, 1, 0, 1, 2, 3, 4, 0, 0, 3, 0, 0, 0, 4, 0]
},
{
  // Write over with empty buffer
  writes: [
  { bufferOffset: 0, data: kTestData, arrayType: 'Uint8Array', useArrayBuffer: false },
  { bufferOffset: 0, data: [], arrayType: 'Uint8Array', useArrayBuffer: false }]
  // Expected [0, 1, 2, 3, 4, 5 ,6 ,7, 8, 9, 10, 11, 12, 13, 14, 15]
},
{
  // Zero buffer
  writes: [{ bufferOffset: 0, data: [], arrayType: 'Uint8Array', useArrayBuffer: false }]
}, // Expected []
{
  // Unaligned source
  writes: [
  {
    bufferOffset: 0,
    data: [0x77, ...kTestData],
    arrayType: 'Uint8Array',
    useArrayBuffer: false,
    dataOffset: 1
  }]
  // Expected [0, 1, 2, 3, 4, 5 ,6 ,7, 8, 9, 10, 11, 12, 13, 14, 15]
},
{
  // Multiple overlapping writes
  writes: [
  {
    bufferOffset: 0,
    data: [0x05050505, 0x05050505, 0x05050505, 0x05050505, 0x05050505],
    arrayType: 'Uint32Array',
    useArrayBuffer: false
  },
  {
    bufferOffset: 0,
    data: [0x04040404, 0x04040404, 0x04040404, 0x04040404],
    arrayType: 'Uint32Array',
    useArrayBuffer: false
  },
  {
    bufferOffset: 0,
    data: [0x03030303, 0x03030303, 0x03030303],
    arrayType: 'Uint32Array',
    useArrayBuffer: false
  },
  {
    bufferOffset: 0,
    data: [0x02020202, 0x02020202],
    arrayType: 'Uint32Array',
    useArrayBuffer: false
  },
  {
    bufferOffset: 0,
    data: [0x01010101],
    arrayType: 'Uint32Array',
    useArrayBuffer: false
  }]
  // Expected [1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5]
}]
).
fn((t) => {
  t.testWriteBuffer(...t.params.writes);
});