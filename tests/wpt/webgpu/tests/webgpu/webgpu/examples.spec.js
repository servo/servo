/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Examples of writing CTS tests with various features.

Start here when looking for examples of basic framework usage.
`;import { makeTestGroup } from '../common/framework/test_group.js';

import { GPUTest } from './gpu_test.js';

// To run these tests in the standalone runner, run `npm start` then open:
// - http://localhost:XXXX/standalone/?runnow=1&q=webgpu:examples:*
// To run in WPT, copy/symlink the out-wpt/ directory as the webgpu/ directory in WPT, then open:
// - (wpt server url)/webgpu/cts.https.html?q=webgpu:examples:
//
// Tests here can be run individually or in groups:
// - ?q=webgpu:examples:basic,async:
// - ?q=webgpu:examples:basic,async:*
// - ?q=webgpu:examples:basic,*
// - ?q=webgpu:examples:*

export const g = makeTestGroup(GPUTest);

// Note: spaces aren't allowed in test names; use underscores.
g.test('test_name').fn((_t) => {});

g.test('not_implemented_yet,without_plan').unimplemented();
g.test('not_implemented_yet,with_plan').
desc(
  `
Plan for this test. What it tests. Summary of how it tests that functionality.
- Description of cases, by describing parameters {a, b, c}
- x= more parameters {x, y, z}
`
).
unimplemented();

g.test('basic').fn((t) => {
  t.expect(true);
  t.expect(true, 'true should be true');

  t.shouldThrow(
    // The expected '.name' of the thrown error.
    'TypeError',
    // This function is run inline inside shouldThrow, and is expected to throw.
    () => {
      throw new TypeError();
    },
    // Log message.
    { message: 'function should throw Error' }
  );
});

g.test('basic,async').fn((t) => {
  // shouldReject must be awaited to ensure it can wait for the promise before the test ends.
  t.shouldReject(
    // The expected '.name' of the thrown error.
    'TypeError',
    // Promise expected to reject.
    Promise.reject(new TypeError()),
    // Log message.
    { message: 'Promise.reject should reject' }
  );

  // Promise can also be an IIFE (immediately-invoked function expression).
  t.shouldReject(
    'TypeError',

    (async () => {
      throw new TypeError();
    })(),
    { message: 'Promise.reject should reject' }
  );
});

g.test('basic,plain_cases').
desc(
  `
A test can be parameterized with a simple array of objects using .paramsSimple([ ... ]).
Each such instance of the test is a "case".

In this example, the following cases are generated (identified by their "query string"),
each with just one subcase:
  - webgpu:examples:basic,cases:x=2;y=2      runs 1 subcase, with t.params set to:
      - { x:   2, y:   2 }
  - webgpu:examples:basic,cases:x=-10;y=-10  runs 1 subcase, with t.params set to:
      - { x: -10, y: -10 }
  `
).
paramsSimple([
{ x: 2, y: 2 }, //
{ x: -10, y: -10 }]
).
fn((t) => {
  t.expect(t.params.x === t.params.y);
});

g.test('basic,plain_cases_private').
desc(
  `
Parameters can be public ("x", "y") which means they're part of the case name.
They can also be private by starting with an underscore ("_result"), which passes
them into the test but does not make them part of the case name:

In this example, the following cases are generated, each with just one subcase:
  - webgpu:examples:basic,cases:x=2;y=4     runs 1 subcase, with t.params set to:
      - { x:   2, y:  4, _result: 6 }
  - webgpu:examples:basic,cases:x=-10;y=18  runs 1 subcase, with t.params set to:
      - { x: -10, y: 18, _result: 8 }
  `
).
paramsSimple([
{ x: 2, y: 4, _result: 6 }, //
{ x: -10, y: 18, _result: 8 }]
).
fn((t) => {
  t.expect(t.params.x + t.params.y === t.params._result);
});
// (note the blank comment above to enforce newlines on autoformat)

g.test('basic,builder_cases').
desc(
  `
A "CaseParamsBuilder" or "SubcaseParamsBuilder" can be passed to .params() instead.
The params builder provides facilities for generating tests combinatorially (by cartesian
product). For convenience, the "unit" CaseParamsBuilder is passed as an argument ("u" below).

In this example, the following cases are generated, each with just one subcase:
  - webgpu:examples:basic,cases:x=1,y=1  runs 1 subcase, with t.params set to:
      - { x: 1, y: 1 }
  - webgpu:examples:basic,cases:x=1,y=2  runs 1 subcase, with t.params set to:
      - { x: 1, y: 2 }
  - webgpu:examples:basic,cases:x=2,y=1  runs 1 subcase, with t.params set to:
      - { x: 2, y: 1 }
  - webgpu:examples:basic,cases:x=2,y=2  runs 1 subcase, with t.params set to:
      - { x: 2, y: 2 }
  `
).
params((u) =>
u //
.combineWithParams([{ x: 1 }, { x: 2 }]).
combineWithParams([{ y: 1 }, { y: 2 }])
).
fn(() => {});

g.test('basic,builder_cases_subcases').
desc(
  `
Each case sub-parameterized using .beginSubcases().
Each such instance of the test is a "subcase", which cannot be run independently of other
subcases. It is somewhat like wrapping the entire fn body in a for-loop.

In this example, the following cases are generated:
  - webgpu:examples:basic,cases:x=1      runs 2 subcases, with t.params set to:
      - { x: 1, y: 1 }
      - { x: 1, y: 2 }
  - webgpu:examples:basic,cases:x=2      runs 2 subcases, with t.params set to:
      - { x: 2, y: 1 }
      - { x: 2, y: 2 }
  `
).
params((u) =>
u //
.combineWithParams([{ x: 1 }, { x: 2 }]).
beginSubcases().
combineWithParams([{ y: 1 }, { y: 2 }])
).
fn(() => {});

g.test('basic,builder_subcases').
desc(
  `
In this example, the following single case is generated:
  - webgpu:examples:basic,cases:         runs 4 subcases, with t.params set to:
      - { x: 1, y: 1 }
      - { x: 1, y: 2 }
      - { x: 2, y: 1 }
      - { x: 2, y: 2 }
  `
).
params((u) =>
u //
.beginSubcases().
combineWithParams([{ x: 1 }, { x: 2 }]).
combineWithParams([{ y: 1 }, { y: 2 }])
).
fn(() => {});

g.test('basic,builder_subcases_short').
desc(
  `
As a shorthand, .paramsSubcasesOnly() can be used.

In this example, the following single case is generated:
  - webgpu:examples:basic,cases:         runs 4 subcases, with t.params set to:
      - { x: 1, y: 1 }
      - { x: 1, y: 2 }
      - { x: 2, y: 1 }
      - { x: 2, y: 2 }
  `
).
paramsSubcasesOnly((u) =>
u //
.combineWithParams([{ x: 1 }, { x: 2 }]).
combineWithParams([{ y: 1 }, { y: 2 }])
).
fn(() => {});

g.test('gpu,async').fn(async (t) => {
  const x = await t.queue.onSubmittedWorkDone();
  t.expect(x === undefined);
});

g.test('gpu,buffers').fn((t) => {
  const data = new Uint32Array([0, 1234, 0]);
  const src = t.makeBufferWithContents(data, GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST);

  // Use the expectGPUBufferValuesEqual helper to check the actual contents of a GPUBuffer.
  // This makes a copy and then asynchronously checks the contents. The test fixture will
  // wait on that result before reporting whether the test passed or failed.
  t.expectGPUBufferValuesEqual(src, data);
});

// One of the following two tests should be skipped on most platforms.

g.test('gpu,with_texture_compression,bc').
desc(
  `Example of a test using a device descriptor.
Tests that a BC format passes validation iff the feature is enabled.`
).
params((u) => u.combine('textureCompressionBC', [false, true])).
beforeAllSubcases((t) => {
  const { textureCompressionBC } = t.params;

  if (textureCompressionBC) {
    t.selectDeviceOrSkipTestCase('texture-compression-bc');
  }
}).
fn((t) => {
  const { textureCompressionBC } = t.params;
  const shouldError = !textureCompressionBC;
  t.shouldThrow(shouldError ? 'TypeError' : false, () => {
    t.createTextureTracked({
      format: 'bc1-rgba-unorm',
      size: [4, 4, 1],
      usage: GPUTextureUsage.TEXTURE_BINDING
    });
  });
});

g.test('gpu,with_texture_compression,etc2').
desc(
  `Example of a test using a device descriptor.
Tests that an ETC2 format passes validation iff the feature is enabled.`
).
params((u) => u.combine('textureCompressionETC2', [false, true])).
beforeAllSubcases((t) => {
  const { textureCompressionETC2 } = t.params;

  if (textureCompressionETC2) {
    t.selectDeviceOrSkipTestCase('texture-compression-etc2');
  }
}).
fn((t) => {
  const { textureCompressionETC2 } = t.params;

  const shouldError = !textureCompressionETC2;
  t.shouldThrow(shouldError ? 'TypeError' : false, () => {
    t.createTextureTracked({
      format: 'etc2-rgb8unorm',
      size: [4, 4, 1],
      usage: GPUTextureUsage.TEXTURE_BINDING
    });
  });
});