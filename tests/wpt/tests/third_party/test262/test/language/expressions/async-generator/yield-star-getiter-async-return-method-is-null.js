// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
description: >
  If iterator's "return" method is `null`,
  received completion is forwarded to the runtime.
info: |
  YieldExpression : yield * AssignmentExpression

  [...]
  7. Repeat,
    [...]
    c. Else,
      i. Assert: received.[[Type]] is return.
      ii. Let return be ? GetMethod(iterator, "return").
      iii. If return is undefined, then
        [...]
        2. Return Completion(received).

  GetMethod ( V, P )

  [...]
  2. Let func be ? GetV(V, P).
  3. If func is either undefined or null, return undefined.
features: [Symbol.asyncIterator, async-iteration]
flags: [async]
---*/

var returnGets = 0;
var asyncIterable = {
  [Symbol.asyncIterator]: function() {
    return this;
  },
  next: function() {
    return {value: 1, done: false};
  },
  get return() {
    returnGets += 1;
    return null;
  },
};

async function* asyncGenerator() {
  yield* asyncIterable;
}

var asyncIterator = asyncGenerator();
asyncIterator.next().then(function() {
  return asyncIterator.return(2).then(function(result) {
    assert.sameValue(result.value, 2);
    assert.sameValue(result.done, true);
    assert.sameValue(returnGets, 1);
  });
}).then($DONE, $DONE);
