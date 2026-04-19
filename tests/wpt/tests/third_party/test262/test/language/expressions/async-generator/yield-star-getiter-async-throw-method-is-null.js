// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
description: >
  If iterator's "throw" method is `null`,
  AsyncIteratorClose is called before rising TypeError.
info: |
  YieldExpression : yield * AssignmentExpression

  [...]
  7. Repeat,
    [...]
    b. Else if received.[[Type]] is throw, then
      i. Let throw be ? GetMethod(iterator, "throw").
      ii. If throw is not undefined, then
        [...]
      iii. Else,
        [...]
        3. If generatorKind is async, perform ? AsyncIteratorClose(iteratorRecord, closeCompletion).
        [...]
        6. Throw a TypeError exception.

  GetMethod ( V, P )

  [...]
  2. Let func be ? GetV(V, P).
  3. If func is either undefined or null, return undefined.

  AsyncIteratorClose ( iteratorRecord, completion )

  [...]
  4. Let innerResult be GetMethod(iterator, "return").
  5. If innerResult.[[Type]] is normal, then
    a. Let return be innerResult.[[Value]].
    b. If return is undefined, return Completion(completion).
features: [Symbol.asyncIterator, async-iteration]
flags: [async]
---*/

var throwGets = 0;
var returnGets = 0;
var asyncIterable = {
  [Symbol.asyncIterator]: function() {
    return this;
  },
  next: function() {
    return {value: 1, done: false};
  },
  get throw() {
    throwGets += 1;
    return null;
  },
  get return() {
    returnGets += 1;
  },
};

async function* asyncGenerator() {
  yield* asyncIterable;
}

var asyncIterator = asyncGenerator();
asyncIterator.next().then(function() {
  return asyncIterator.throw();
}).then(function(result) {
  throw new Test262Error("Promise should be rejected, got: " + result.value);
}, function(err) {
  assert.sameValue(err.constructor, TypeError);
  assert.sameValue(throwGets, 1);
  assert.sameValue(returnGets, 1);
}).then($DONE, $DONE);
