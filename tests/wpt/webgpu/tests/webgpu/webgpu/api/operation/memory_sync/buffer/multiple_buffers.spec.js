/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Memory Synchronization Tests for multiple buffers: read before write, read after write, and write after write.

- Create multiple src buffers and initialize it to 0, wait on the fence to ensure the data is initialized.
Write Op: write a value (say 1) into the src buffer via render pass, compute pass, copy, write buffer, etc.
Read Op: read the value from the src buffer and write it to dst buffer via render pass (vertex, index, indirect input, uniform, storage), compute pass, copy etc.
Wait on another fence, then call expectContents to verify the dst buffer value.
  - x= write op: {storage buffer in {compute, render, render-via-bundle}, t2b copy dst, b2b copy dst, writeBuffer}
  - x= read op: {index buffer, vertex buffer, indirect buffer (draw, draw indexed, dispatch), uniform buffer, {readonly, readwrite} storage buffer in {compute, render, render-via-bundle}, b2b copy src, b2t copy src}
  - x= read-write sequence: {read then write, write then read, write then write}
  - x= op context: {queue, command-encoder, compute-pass-encoder, render-pass-encoder, render-bundle-encoder}, x= op boundary: {queue-op, command-buffer, pass, execute-bundles, render-bundle}
    - Not every context/boundary combinations are valid. We have the checkOpsValidForContext func to do the filtering.
  - If two writes are in the same passes, render result has loose guarantees.

TODO: Tests with more than one buffer to try to stress implementations a little bit more.
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import {
  kOperationBoundaries,
  kBoundaryInfo,
  OperationContextHelper,
} from '../operation_context_helper.js';

import {
  kAllReadOps,
  kAllWriteOps,
  BufferSyncTest,
  checkOpsValidForContext,
} from './buffer_sync_test.js';

// The src value is what stores in the src buffer before any operation.
const kSrcValue = 0;
// The op value is what the read/write operation write into the target buffer.
const kOpValue = 1;

export const g = makeTestGroup(BufferSyncTest);

g.test('rw')
  .desc(
    `
    Perform a 'read' operations on multiple buffers, followed by a 'write' operation.
    Operations are separated by a 'boundary' (pass, encoder, queue-op, etc.).
    Test that the results are synchronized.
    The read should not see the contents written by the subsequent write.
  `
  )
  .params(u =>
    u //
      .combine('boundary', kOperationBoundaries)
      .expand('_context', p => kBoundaryInfo[p.boundary].contexts)
      .expandWithParams(function* ({ _context }) {
        for (const readOp of kAllReadOps) {
          for (const writeOp of kAllWriteOps) {
            if (checkOpsValidForContext([readOp, writeOp], _context)) {
              yield {
                readOp,
                readContext: _context[0],
                writeOp,
                writeContext: _context[1],
              };
            }
          }
        }
      })
  )
  .fn(async t => {
    const { readContext, readOp, writeContext, writeOp, boundary } = t.params;
    const helper = new OperationContextHelper(t);

    const srcBuffers = [];
    const dstBuffers = [];

    const kBufferCount = 4;
    for (let i = 0; i < kBufferCount; i++) {
      const { srcBuffer, dstBuffer } = await t.createBuffersForReadOp(readOp, kSrcValue, kOpValue);
      srcBuffers.push(srcBuffer);
      dstBuffers.push(dstBuffer);
    }

    await t.createIntermediateBuffersAndTexturesForWriteOp(writeOp, 0, kOpValue);

    // The read op will read from src buffers and write to dst buffers based on what it reads.
    // A boundary will separate multiple read and write operations. The write op will write the
    // given op value into each src buffer as well. The write op happens after read op. So we are
    // expecting each src value to be in the mapped dst buffer.
    for (let i = 0; i < kBufferCount; i++) {
      t.encodeReadOp(helper, readOp, readContext, srcBuffers[i], dstBuffers[i]);
    }

    helper.ensureBoundary(boundary);

    for (let i = 0; i < kBufferCount; i++) {
      t.encodeWriteOp(helper, writeOp, writeContext, srcBuffers[i], 0, kOpValue);
    }

    helper.ensureSubmit();

    for (let i = 0; i < kBufferCount; i++) {
      // Only verify the value of the first element of each dstBuffer.
      t.verifyData(dstBuffers[i], kSrcValue);
    }
  });

g.test('wr')
  .desc(
    `
    Perform a 'write' operation on on multiple buffers, followed by a 'read' operation.
    Operations are separated by a 'boundary' (pass, encoder, queue-op, etc.).
    Test that the results are synchronized.
    The read should see exactly the contents written by the previous write.`
  )
  .params(u =>
    u //
      .combine('boundary', kOperationBoundaries)
      .expand('_context', p => kBoundaryInfo[p.boundary].contexts)
      .expandWithParams(function* ({ _context }) {
        for (const readOp of kAllReadOps) {
          for (const writeOp of kAllWriteOps) {
            if (checkOpsValidForContext([readOp, writeOp], _context)) {
              yield {
                readOp,
                readContext: _context[0],
                writeOp,
                writeContext: _context[1],
              };
            }
          }
        }
      })
  )
  .fn(async t => {
    const { readContext, readOp, writeContext, writeOp, boundary } = t.params;
    const helper = new OperationContextHelper(t);

    const srcBuffers = [];
    const dstBuffers = [];

    const kBufferCount = 4;

    for (let i = 0; i < kBufferCount; i++) {
      const { srcBuffer, dstBuffer } = await t.createBuffersForReadOp(readOp, kSrcValue, kOpValue);

      srcBuffers.push(srcBuffer);
      dstBuffers.push(dstBuffer);
    }

    await t.createIntermediateBuffersAndTexturesForWriteOp(writeOp, 0, kOpValue);

    // The write op will write the given op value into src buffers.
    // The read op will read from src buffers and write to dst buffers based on what it reads.
    // The write op happens before read op. So we are expecting the op value to be in the dst
    // buffers.
    for (let i = 0; i < kBufferCount; i++) {
      t.encodeWriteOp(helper, writeOp, writeContext, srcBuffers[i], 0, kOpValue);
    }

    helper.ensureBoundary(boundary);

    for (let i = 0; i < kBufferCount; i++) {
      t.encodeReadOp(helper, readOp, readContext, srcBuffers[i], dstBuffers[i]);
    }

    helper.ensureSubmit();

    for (let i = 0; i < kBufferCount; i++) {
      // Only verify the value of the first element of the dstBuffer
      t.verifyData(dstBuffers[i], kOpValue);
    }
  });

g.test('ww')
  .desc(
    `
    Perform a 'first' write operation on multiple buffers, followed by a 'second' write operation.
    Operations are separated by a 'boundary' (pass, encoder, queue-op, etc.).
    Test that the results are synchronized.
    The second write should overwrite the contents of the first.`
  )
  .params(u =>
    u //
      .combine('boundary', kOperationBoundaries)
      .expand('_context', p => kBoundaryInfo[p.boundary].contexts)
      .expandWithParams(function* ({ _context }) {
        for (const firstWriteOp of kAllWriteOps) {
          for (const secondWriteOp of kAllWriteOps) {
            if (checkOpsValidForContext([firstWriteOp, secondWriteOp], _context)) {
              yield {
                writeOps: [firstWriteOp, secondWriteOp],
                contexts: _context,
              };
            }
          }
        }
      })
  )
  .fn(async t => {
    const { writeOps, contexts, boundary } = t.params;
    const helper = new OperationContextHelper(t);

    const buffers = [];

    const kBufferCount = 4;

    for (let i = 0; i < kBufferCount; i++) {
      const buffer = await t.createBufferWithValue(0);

      buffers.push(buffer);
    }

    await t.createIntermediateBuffersAndTexturesForWriteOp(writeOps[0], 0, 1);
    await t.createIntermediateBuffersAndTexturesForWriteOp(writeOps[1], 1, 2);

    for (let i = 0; i < kBufferCount; i++) {
      t.encodeWriteOp(helper, writeOps[0], contexts[0], buffers[i], 0, 1);
    }

    helper.ensureBoundary(boundary);

    for (let i = 0; i < kBufferCount; i++) {
      t.encodeWriteOp(helper, writeOps[1], contexts[1], buffers[i], 1, 2);
    }

    helper.ensureSubmit();

    for (let i = 0; i < kBufferCount; i++) {
      t.verifyData(buffers[i], 2);
    }
  });

g.test('multiple_pairs_of_draws_in_one_render_pass')
  .desc(
    `
  Test write-after-write operations on multiple buffers via the one render pass. The first write
  will write the buffer index * 2 + 1 into all storage buffers. The second write will write the
  buffer index * 2 + 2 into the all buffers in the same pass. Expected data in all buffers is either
  buffer index * 2 + 1 or buffer index * 2 + 2. It may use bundle in each draw.
  `
  )
  .paramsSubcasesOnly(u =>
    u //
      .combine('firstDrawUseBundle', [false, true])
      .combine('secondDrawUseBundle', [false, true])
  )
  .fn(async t => {
    const { firstDrawUseBundle, secondDrawUseBundle } = t.params;

    const encoder = t.device.createCommandEncoder();
    const passEncoder = t.beginSimpleRenderPass(encoder);

    const kBufferCount = 4;
    const buffers = [];
    for (let b = 0; b < kBufferCount; ++b) {
      const buffer = await t.createBufferWithValue(0);
      buffers.push(buffer);

      const useBundle = [firstDrawUseBundle, secondDrawUseBundle];
      for (let i = 0; i < 2; ++i) {
        const renderEncoder = useBundle[i]
          ? t.device.createRenderBundleEncoder({
              colorFormats: ['rgba8unorm'],
            })
          : passEncoder;
        const pipeline = t.createStorageWriteRenderPipeline(2 * b + i + 1);
        const bindGroup = t.createBindGroup(pipeline, buffer);
        renderEncoder.setPipeline(pipeline);
        renderEncoder.setBindGroup(0, bindGroup);
        renderEncoder.draw(1, 1, 0, 0);
        if (useBundle[i]) passEncoder.executeBundles([renderEncoder.finish()]);
      }
    }

    passEncoder.end();
    t.device.queue.submit([encoder.finish()]);
    for (let b = 0; b < kBufferCount; ++b) {
      t.verifyDataTwoValidValues(buffers[b], 2 * b + 1, 2 * b + 2);
    }
  });

g.test('multiple_pairs_of_draws_in_one_render_bundle')
  .desc(
    `
  Test write-after-write operations on multiple buffers via the one render bundle. The first write
  will write the buffer index * 2 + 1 into all storage buffers. The second write will write the
  buffer index * 2 + 2 into the all buffers in the same pass. Expected data in all buffers is either
  buffer index * 2 + 1 or buffer index * 2 + 2.
  `
  )
  .fn(async t => {
    const encoder = t.device.createCommandEncoder();
    const passEncoder = t.beginSimpleRenderPass(encoder);
    const renderEncoder = t.device.createRenderBundleEncoder({
      colorFormats: ['rgba8unorm'],
    });

    const kBufferCount = 4;
    const buffers = [];
    for (let b = 0; b < kBufferCount; ++b) {
      const buffer = await t.createBufferWithValue(0);
      buffers.push(buffer);

      for (let i = 0; i < 2; ++i) {
        const pipeline = t.createStorageWriteRenderPipeline(2 * b + i + 1);
        const bindGroup = t.createBindGroup(pipeline, buffer);
        renderEncoder.setPipeline(pipeline);
        renderEncoder.setBindGroup(0, bindGroup);
        renderEncoder.draw(1, 1, 0, 0);
      }
    }

    passEncoder.executeBundles([renderEncoder.finish()]);
    passEncoder.end();
    t.device.queue.submit([encoder.finish()]);
    for (let b = 0; b < kBufferCount; ++b) {
      t.verifyDataTwoValidValues(buffers[b], 2 * b + 1, 2 * b + 2);
    }
  });

g.test('multiple_pairs_of_dispatches_in_one_compute_pass')
  .desc(
    `
  Test write-after-write operations on multiple buffers via the one compute pass. The first write
  will write the buffer index * 2 + 1 into all storage buffers. The second write will write the
  buffer index * 2 + 2 into the all buffers in the same pass. Expected data in all buffers is the
  buffer index * 2 + 2.
  `
  )
  .fn(async t => {
    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginComputePass();

    const kBufferCount = 4;
    const buffers = [];
    for (let b = 0; b < kBufferCount; ++b) {
      const buffer = await t.createBufferWithValue(0);
      buffers.push(buffer);

      for (let i = 0; i < 2; ++i) {
        const pipeline = t.createStorageWriteComputePipeline(2 * b + i + 1);
        const bindGroup = t.createBindGroup(pipeline, buffer);
        pass.setPipeline(pipeline);
        pass.setBindGroup(0, bindGroup);
        pass.dispatchWorkgroups(1);
      }
    }

    pass.end();

    t.device.queue.submit([encoder.finish()]);
    for (let b = 0; b < kBufferCount; ++b) {
      t.verifyData(buffers[b], 2 * b + 2);
    }
  });
