/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Tests for the behavior of ArrayBuffers returned by getMappedRange.

TODO: Add tests that transfer to another thread instead of just using MessageChannel.
TODO: Add tests for any other Web APIs that can detach ArrayBuffers.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { timeout } from '../../../../common/util/timeout.js';
import { GPUTest } from '../../../gpu_test.js';
import { checkElementsEqual } from '../../../util/check_contents.js';

export const g = makeTestGroup(GPUTest);

g.test('postMessage')
  .desc(
    `Using postMessage to send a getMappedRange-returned ArrayBuffer throws a TypeError
     if it was included in the transfer list. Otherwise, it makes a copy.
    Test combinations of transfer={false, true}, mapMode={read,write}.`
  )
  .params(u =>
    u //
      .combine('transfer', [false, true])
      .combine('mapMode', ['READ', 'WRITE'])
  )
  .fn(async t => {
    const { transfer, mapMode } = t.params;
    const kSize = 1024;

    // Populate initial data.
    const initialData = new Uint32Array(new ArrayBuffer(kSize));
    for (let i = 0; i < initialData.length; ++i) {
      initialData[i] = i;
    }

    const buf = t.makeBufferWithContents(
      initialData,
      mapMode === 'WRITE' ? GPUBufferUsage.MAP_WRITE : GPUBufferUsage.MAP_READ
    );

    await buf.mapAsync(GPUMapMode[mapMode]);
    const ab1 = buf.getMappedRange();
    t.expect(ab1.byteLength === kSize, 'ab1 should have the size of the buffer');

    const mc = new MessageChannel();
    const ab2Promise = new Promise(resolve => {
      mc.port2.onmessage = ev => {
        if (transfer) {
          t.fail(
            `postMessage with ab1 in transfer list should not be received. Unexpected message: ${ev.data}`
          );
        } else {
          resolve(ev.data);
        }
      };
    });

    if (transfer) {
      t.shouldThrow('TypeError', () => mc.port1.postMessage(ab1, [ab1]));
      // Wait to make sure the postMessage isn't received.
      await new Promise(resolve => timeout(resolve, 100));
    } else {
      mc.port1.postMessage(ab1);
    }
    t.expect(ab1.byteLength === kSize, 'after postMessage, ab1 should not be detached');

    if (!transfer) {
      const ab2 = await ab2Promise;
      t.expect(ab2.byteLength === kSize, 'ab2 should be the same size');
      const ab2Data = new Uint32Array(ab2, 0, initialData.length);
      // ab2 should have the same initial contents.
      t.expectOK(checkElementsEqual(ab2Data, initialData));

      // Mutations to ab2 should not be visible in ab1.
      const ab1Data = new Uint32Array(ab1, 0, initialData.length);
      const abs2NewData = initialData.slice().reverse();
      for (let i = 0; i < ab2Data.length; ++i) {
        ab2Data[i] = abs2NewData[i];
      }
      t.expectOK(checkElementsEqual(ab1Data, initialData));
      t.expectOK(checkElementsEqual(ab2Data, abs2NewData));
    }

    buf.unmap();
    t.expect(ab1.byteLength === 0, 'after unmap, ab1 should be detached');

    // Transferring an already-detached ArrayBuffer is a DataCloneError.
    t.shouldThrow('DataCloneError', () => mc.port1.postMessage(ab1, [ab1]));
  });
