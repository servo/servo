/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
API operations tests for clearBuffer.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('clear')
  .desc(
    `Validate the correctness of the clear by filling the srcBuffer with testable data, doing
  clearBuffer(), and verifying the content of the whole srcBuffer with MapRead:
  Clear {4 bytes, part of, the whole} buffer {with, without} a non-zero valid offset that
  - covers the whole buffer
  - covers the beginning of the buffer
  - covers the end of the buffer
  - covers neither the beginning nor the end of the buffer`
  )
  .paramsSubcasesOnly(u =>
    u //
      .combine('offset', [0, 4, 8, 16, undefined])
      .combine('size', [0, 4, 8, 16, undefined])
      .expand('bufferSize', p => [
        (p.offset ?? 0) + (p.size ?? 16),
        (p.offset ?? 0) + (p.size ?? 16) + 8,
      ])
  )
  .fn(t => {
    const { offset, size, bufferSize } = t.params;

    const bufferData = new Uint8Array(bufferSize);
    for (let i = 0; i < bufferSize; ++i) {
      bufferData[i] = i + 1;
    }

    const buffer = t.makeBufferWithContents(
      bufferData,
      GPUBufferUsage.COPY_DST | GPUBufferUsage.COPY_SRC
    );

    const encoder = t.device.createCommandEncoder();
    encoder.clearBuffer(buffer, offset, size);
    t.device.queue.submit([encoder.finish()]);

    const expectOffset = offset ?? 0;
    const expectSize = size ?? bufferSize - expectOffset;

    for (let i = 0; i < expectSize; ++i) {
      bufferData[expectOffset + i] = 0;
    }

    t.expectGPUBufferValuesEqual(buffer, bufferData);
  });
