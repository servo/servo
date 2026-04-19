// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-asynciteratorclose
description: >
  If iterator's "return" method is `null`,
  received non-throw completion is forwarded to the runtime.
info: |
  AsyncIteratorClose ( iteratorRecord, completion )

  [...]
  4. Let innerResult be GetMethod(iterator, "return").
  5. If innerResult.[[Type]] is normal,
    a. Let return be innerResult.[[Value]].
    b. If return is undefined, return Completion(completion).

  GetMethod ( V, P )

  [...]
  2. Let func be ? GetV(V, P).
  3. If func is either undefined or null, return undefined.
features: [Symbol.asyncIterator, async-iteration]
flags: [async]
includes: [asyncHelpers.js]
---*/

var iterationCount = 0;
var returnGets = 0;

var iterable = {};
iterable[Symbol.asyncIterator] = function() {
  return {
    next: function() {
      return {value: 1, done: false};
    },
    get return() {
      returnGets += 1;
      return null;
    },
  };
};

asyncTest(async function() {
  for await (var _ of iterable) {
    iterationCount += 1;
    break;
  }

  assert.sameValue(iterationCount, 1);
  assert.sameValue(returnGets, 1);
});
