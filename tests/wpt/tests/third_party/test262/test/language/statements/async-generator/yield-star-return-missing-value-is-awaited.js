// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
description: >
  If iterator's "return" method is missing,
  received value is awaited before forwarded to the runtime.
info: |
  YieldExpression : yield * AssignmentExpression

  [...]
  7. Repeat,
    [...]
    c. Else,
      i. Assert: received.[[Type]] is return.
      ii. Let return be ? GetMethod(iterator, "return").
      iii. If return is undefined, then
        1. If generatorKind is async, then set received.[[Value]] to ? Await(received.[[Value]]).
        2. Return Completion(received).
features: [Symbol.asyncIterator, async-iteration]
flags: [async]
---*/

var asyncIterable = {
  [Symbol.asyncIterator]: function() {
    return this;
  },
  next: function() {
    return {value: 1, done: false};
  },
};

async function* asyncGenerator() {
  yield* asyncIterable;
}

var asyncIterator = asyncGenerator();
asyncIterator.next().then(function() {
  var promise = Promise.resolve(2).then(() => 3);
  return asyncIterator.return(promise).then(function(result) {
    assert.sameValue(result.value, 3);
    assert.sameValue(result.done, true);
  });
}).then($DONE, $DONE);
