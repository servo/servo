// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncblockstart
description: >
  Evaluation of await ticks order
info: |
  AsyncBlockStart ( promiseCapability, asyncBody, asyncContext )

  1. Assert: promiseCapability is a PromiseCapability Record.
  2. Let runningContext be the running execution context.
  3. Set the code evaluation state of asyncContext such that when evaluation is resumed for that execution context the following steps will be performed:
    a. Let result be the result of evaluating asyncBody.
    ...
includes: [compareArray.js]
flags: [module, async]
features: [top-level-await]
---*/

var expected = [
  'tick 1',
  'await 1',
  'tick 2',
  'await 2',
  'tick 3',
  'await 3',
  'tick 4',
  'await 4',
];

var actual = [];

Promise.resolve(0)
  .then(() => actual.push('tick 1'))
  .then(() => actual.push('tick 2'))
  .then(() => actual.push('tick 3'))
  .then(() => actual.push('tick 4'))
  .then(() => {
    assert.compareArray(actual, expected, 'Ticks for top level await and promises');
}).then($DONE, $DONE);

await 1; actual.push('await 1');
await 2; actual.push('await 2');
await 3; actual.push('await 3');
await 4; actual.push('await 4');
