/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Memory Synchronization Tests for Buffer: read before write, read after write, and write after write.

- Create a src buffer and initialize it to 0, wait on the fence to ensure the data is initialized.
Write Op: write a value (say 1) into the src buffer via render pass, compute pass, copy, write buffer, etc.
Read Op: read the value from the src buffer and write it to dst buffer via render pass (vertex, index, indirect input, uniform, storage), compute pass, copy etc.
Wait on another fence, then call expectContents to verify the dst buffer value.
  - x= write op: {storage buffer in {compute, render, render-via-bundle}, t2b copy dst, b2b copy dst, writeBuffer}
  - x= read op: {index buffer, vertex buffer, indirect buffer (draw, draw indexed, dispatch), uniform buffer, {readonly, readwrite} storage buffer in {compute, render, render-via-bundle}, b2b copy src, b2t copy src}
  - x= read-write sequence: {read then write, write then read, write then write}
  - x= op context: {queue, command-encoder, compute-pass-encoder, render-pass-encoder, render-bundle-encoder}, x= op boundary: {queue-op, command-buffer, pass, execute-bundles, render-bundle}
    - Not every context/boundary combinations are valid. We have the checkOpsValidForContext func to do the filtering.
  - If two writes are in the same passes, render result has loose guarantees.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import {
  kOperationBoundaries,
  kBoundaryInfo,
  OperationContextHelper } from
'../operation_context_helper.js';

import {
  kAllReadOps,
  kAllWriteOps,
  BufferSyncTest,
  checkOpsValidForContext } from
'./buffer_sync_test.js';

// The src value is what stores in the src buffer before any operation.
const kSrcValue = 0;
// The op value is what the read/write operation write into the target buffer.
const kOpValue = 1;

export const g = makeTestGroup(BufferSyncTest);

g.test('rw').
desc(
  `
    Perform a 'read' operations on a buffer, followed by a 'write' operation.
    Operations are separated by a 'boundary' (pass, encoder, queue-op, etc.).
    Test that the results are synchronized.
    The read should not see the contents written by the subsequent write.`
).
params((u) =>
u //
.combine('boundary', kOperationBoundaries).
expand('_context', (p) => kBoundaryInfo[p.boundary].contexts).
expandWithParams(function* ({ _context }) {
  for (const readOp of kAllReadOps) {
    for (const writeOp of kAllWriteOps) {
      if (checkOpsValidForContext([readOp, writeOp], _context)) {
        yield {
          readOp,
          readContext: _context[0],
          writeOp,
          writeContext: _context[1]
        };
      }
    }
  }
})
).
fn(async (t) => {
  const { readContext, readOp, writeContext, writeOp, boundary } = t.params;
  const helper = new OperationContextHelper(t);

  const { srcBuffer, dstBuffer } = await t.createBuffersForReadOp(readOp, kSrcValue, kOpValue);
  await t.createIntermediateBuffersAndTexturesForWriteOp(writeOp, 0, kOpValue);

  // The read op will read from src buffer and write to dst buffer based on what it reads.
  // The write op will write the given op value into src buffer as well.
  // The write op happens after read op. So we are expecting the src value to be in the dst buffer.
  t.encodeReadOp(helper, readOp, readContext, srcBuffer, dstBuffer);
  helper.ensureBoundary(boundary);
  t.encodeWriteOp(helper, writeOp, writeContext, srcBuffer, 0, kOpValue);
  helper.ensureSubmit();
  // Only verify the value of the first element of the dstBuffer
  t.verifyData(dstBuffer, kSrcValue);
});

g.test('wr').
desc(
  `
    Perform a 'write' operation on a buffer, followed by a 'read' operation.
    Operations are separated by a 'boundary' (pass, encoder, queue-op, etc.).
    Test that the results are synchronized.
    The read should see exactly the contents written by the previous write.`
).
params((u) =>
u //
.combine('boundary', kOperationBoundaries).
expand('_context', (p) => kBoundaryInfo[p.boundary].contexts).
expandWithParams(function* ({ _context }) {
  for (const readOp of kAllReadOps) {
    for (const writeOp of kAllWriteOps) {
      if (checkOpsValidForContext([readOp, writeOp], _context)) {
        yield {
          readOp,
          readContext: _context[0],
          writeOp,
          writeContext: _context[1]
        };
      }
    }
  }
})
).
fn(async (t) => {
  const { readContext, readOp, writeContext, writeOp, boundary } = t.params;
  const helper = new OperationContextHelper(t);

  const { srcBuffer, dstBuffer } = await t.createBuffersForReadOp(readOp, kSrcValue, kOpValue);
  await t.createIntermediateBuffersAndTexturesForWriteOp(writeOp, 0, kOpValue);

  // The write op will write the given op value into src buffer.
  // The read op will read from src buffer and write to dst buffer based on what it reads.
  // The write op happens before read op. So we are expecting the op value to be in the dst buffer.
  t.encodeWriteOp(helper, writeOp, writeContext, srcBuffer, 0, kOpValue);
  helper.ensureBoundary(boundary);
  t.encodeReadOp(helper, readOp, readContext, srcBuffer, dstBuffer);
  helper.ensureSubmit();
  // Only verify the value of the first element of the dstBuffer
  t.verifyData(dstBuffer, kOpValue);
});

g.test('ww').
desc(
  `
    Perform a 'first' write operation on a buffer, followed by a 'second' write operation.
    Operations are separated by a 'boundary' (pass, encoder, queue-op, etc.).
    Test that the results are synchronized.
    The second write should overwrite the contents of the first.`
).
params((u) =>
u //
.combine('boundary', kOperationBoundaries).
expand('_context', (p) => kBoundaryInfo[p.boundary].contexts).
expandWithParams(function* ({ _context }) {
  for (const firstWriteOp of kAllWriteOps) {
    for (const secondWriteOp of kAllWriteOps) {
      if (checkOpsValidForContext([firstWriteOp, secondWriteOp], _context)) {
        yield {
          writeOps: [firstWriteOp, secondWriteOp],
          contexts: _context
        };
      }
    }
  }
})
).
fn(async (t) => {
  const { writeOps, contexts, boundary } = t.params;
  const helper = new OperationContextHelper(t);

  const buffer = await t.createBufferWithValue(0);
  await t.createIntermediateBuffersAndTexturesForWriteOp(writeOps[0], 0, 1);
  await t.createIntermediateBuffersAndTexturesForWriteOp(writeOps[1], 1, 2);

  t.encodeWriteOp(helper, writeOps[0], contexts[0], buffer, 0, 1);
  helper.ensureBoundary(boundary);
  t.encodeWriteOp(helper, writeOps[1], contexts[1], buffer, 1, 2);
  helper.ensureSubmit();
  t.verifyData(buffer, 2);
});

// Cases with loose render result guarantees.

g.test('two_draws_in_the_same_render_pass').
desc(
  `Test write-after-write operations in the same render pass. The first write will write 1 into
    a storage buffer. The second write will write 2 into the same buffer in the same pass. Expected
    data in buffer is either 1 or 2. It may use bundle in each draw.`
).
paramsSubcasesOnly((u) =>
u //
.combine('firstDrawUseBundle', [false, true]).
combine('secondDrawUseBundle', [false, true])
).
fn(async (t) => {
  const { firstDrawUseBundle, secondDrawUseBundle } = t.params;
  const buffer = await t.createBufferWithValue(0);
  const encoder = t.device.createCommandEncoder();
  const passEncoder = t.beginSimpleRenderPass(encoder);

  const useBundle = [firstDrawUseBundle, secondDrawUseBundle];
  for (let i = 0; i < 2; ++i) {
    const renderEncoder = useBundle[i] ?
    t.device.createRenderBundleEncoder({
      colorFormats: ['rgba8unorm']
    }) :
    passEncoder;
    const pipeline = t.createStorageWriteRenderPipeline(i + 1);
    const bindGroup = t.createBindGroup(pipeline, buffer);
    renderEncoder.setPipeline(pipeline);
    renderEncoder.setBindGroup(0, bindGroup);
    renderEncoder.draw(1, 1, 0, 0);
    if (useBundle[i])
    passEncoder.executeBundles([renderEncoder.finish()]);
  }

  passEncoder.end();
  t.device.queue.submit([encoder.finish()]);
  t.verifyDataTwoValidValues(buffer, 1, 2);
});

g.test('two_draws_in_the_same_render_bundle').
desc(
  `Test write-after-write operations in the same render bundle. The first write will write 1 into
    a storage buffer. The second write will write 2 into the same buffer in the same pass. Expected
    data in buffer is either 1 or 2.`
).
fn(async (t) => {
  const buffer = await t.createBufferWithValue(0);
  const encoder = t.device.createCommandEncoder();
  const passEncoder = t.beginSimpleRenderPass(encoder);
  const renderEncoder = t.device.createRenderBundleEncoder({
    colorFormats: ['rgba8unorm']
  });

  for (let i = 0; i < 2; ++i) {
    const pipeline = t.createStorageWriteRenderPipeline(i + 1);
    const bindGroup = t.createBindGroup(pipeline, buffer);
    renderEncoder.setPipeline(pipeline);
    renderEncoder.setBindGroup(0, bindGroup);
    renderEncoder.draw(1, 1, 0, 0);
  }

  passEncoder.executeBundles([renderEncoder.finish()]);
  passEncoder.end();
  t.device.queue.submit([encoder.finish()]);
  t.verifyDataTwoValidValues(buffer, 1, 2);
});

g.test('two_dispatches_in_the_same_compute_pass').
desc(
  `Test write-after-write operations in the same compute pass. The first write will write 1 into
    a storage buffer. The second write will write 2 into the same buffer in the same pass. Expected
    data in buffer is 2.`
).
fn(async (t) => {
  const buffer = await t.createBufferWithValue(0);
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();

  for (let i = 0; i < 2; ++i) {
    const pipeline = t.createStorageWriteComputePipeline(i + 1);
    const bindGroup = t.createBindGroup(pipeline, buffer);
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.dispatchWorkgroups(1);
  }

  pass.end();
  t.device.queue.submit([encoder.finish()]);
  t.verifyData(buffer, 2);
});