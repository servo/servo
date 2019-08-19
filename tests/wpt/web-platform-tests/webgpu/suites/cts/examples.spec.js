/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
Examples of writing CTS tests with various features.
`;
import { TestGroup } from '../../framework/index.js';
import { GPUTest } from './gpu_test.js'; // To run these tests in the standalone runner, run `grunt build` or `grunt pre` then open:
// - http://localhost:8080/?runnow=1&q=cts:examples:
// To run in WPT, copy/symlink the out-wpt/ directory as the webgpu/ directory in WPT, then open:
// - (wpt server url)/webgpu/cts.html?q=cts:examples:
//
// Tests here can be run individually or in groups:
// - ?q=cts:examples:basic/async=
// - ?q=cts:examples:basic/
// - ?q=cts:examples:

export const g = new TestGroup(GPUTest);
g.test('basic', t => {
  t.expect(true);
  t.expect(true, 'true should be true');
  t.shouldThrow( // The expected '.name' of the thrown error.
  'TypeError', // This function is run inline inside shouldThrow, and is expected to throw.
  () => {
    throw new TypeError();
  }, // Log message.
  'function should throw Error');
});
g.test('basic/async', async t => {
  // shouldReject must be awaited to ensure it can wait for the promise before the test ends.
  await t.shouldReject( // The expected '.name' of the thrown error.
  'TypeError', // Promise expected to reject.
  Promise.reject(new TypeError()), // Log message.
  'Promise.reject should reject'); // Promise can also be an IIFE.

  await t.shouldReject('TypeError', (async () => {
    throw new TypeError();
  })(), 'Promise.reject should reject');
});
g.test('basic/params', t => {
  t.expect(t.params.x + t.params.y === t.params.result);
}).params([{
  x: 2,
  y: 4,
  result: 6
}, // (blank comment to enforce newlines on autoformat)
{
  x: -10,
  y: 18,
  result: 8
}]);
g.test('gpu/async', async t => {
  const fence = t.queue.createFence();
  t.queue.signal(fence, 2);
  await fence.onCompletion(1);
  t.expect(fence.getCompletedValue() === 2);
});
g.test('gpu/buffers', async t => {
  const data = new Uint32Array([0, 1234, 0]);
  const [src, map] = t.device.createBufferMapped({
    size: 12,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });
  new Uint32Array(map).set(data);
  src.unmap(); // Use the expectContents helper to check the actual contents of a GPUBuffer.
  // Like shouldReject, it must be awaited.

  await t.expectContents(src, data);
});
//# sourceMappingURL=examples.spec.js.map