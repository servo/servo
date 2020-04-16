/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
Examples of writing CTS tests with various features.

Start here when looking for examples of basic framework usage.
`;
import { TestGroup } from '../common/framework/test_group.js';
import { GPUTest } from './gpu_test.js'; // To run these tests in the standalone runner, run `grunt build` or `grunt pre` then open:
// - http://localhost:8080/?runnow=1&q=webgpu:examples:
// To run in WPT, copy/symlink the out-wpt/ directory as the webgpu/ directory in WPT, then open:
// - (wpt server url)/webgpu/cts.html?q=webgpu:examples:
//
// Tests here can be run individually or in groups:
// - ?q=webgpu:examples:basic/async=
// - ?q=webgpu:examples:basic/
// - ?q=webgpu:examples:

export const g = new TestGroup(GPUTest); // Note: spaces in test names are replaced with underscores: webgpu:examples:test_name=

g.test('test name', t => {});
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
  t.shouldReject( // The expected '.name' of the thrown error.
  'TypeError', // Promise expected to reject.
  Promise.reject(new TypeError()), // Log message.
  'Promise.reject should reject'); // Promise can also be an IIFE.

  t.shouldReject('TypeError', (async () => {
    throw new TypeError();
  })(), 'Promise.reject should reject');
}); // A test can be parameterized with a simple array of objects.
//
// Parameters can be public (x, y) which means they're part of the case name.
// They can also be private by starting with an underscore (_result), which passes
// them into the test but does not make them part of the case name:
//
// - webgpu:examples:basic/params={"x":2,"y":4}    runs with t.params = {x: 2, y: 5, _result: 6}.
// - webgpu:examples:basic/params={"x":-10,"y":18} runs with t.params = {x: -10, y: 18, _result: 8}.

g.test('basic/params', t => {
  t.expect(t.params.x + t.params.y === t.params._result);
}).params([{
  x: 2,
  y: 4,
  _result: 6
}, //
{
  x: -10,
  y: 18,
  _result: 8
}]); // (note the blank comment above to enforce newlines on autoformat)

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

  t.expectContents(src, data);
});
//# sourceMappingURL=examples.spec.js.map