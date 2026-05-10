// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
description: >
  `value` property is not accessed when iteration is complete
info: |
  YieldExpression : yield * AssignmentExpression

  1. Let exprRef be the result of evaluating AssignmentExpression.
  2. Let value be ? GetValue(exprRef).
  3. Let iterator be ? GetIterator(value).
  4. Let received be NormalCompletion(undefined).
  5. Repeat
     a. If received.[[Type]] is normal, then
        [...]
     b. Else if received.[[Type]] is throw, then
        i. Let throw be ? GetMethod(iterator, "throw").
        ii. If throw is not undefined, then
            1. Let innerResult be ? Call(throw, iterator, « received.[[Value]]
               »).
            2. NOTE: Exceptions from the inner iterator throw method are
               propagated. Normal completions from an inner throw method are
               processed similarly to an inner next.
            3. If Type(innerResult) is not Object, throw a TypeError exception.
            4. Let done be ? IteratorComplete(innerResult).
            5. If done is true, then
               a. Return ? IteratorValue(innerResult).

  7.4.3 IteratorComplete

  1. Assert: Type(iterResult) is Object.
  2. Return ToBoolean(? Get(iterResult, "done")).
features: [generators, Symbol.iterator]
---*/

var badIter = {};
var callCount = 0;
var spyValue = Object.defineProperty({ done: false }, 'value', {
  get: function() {
    callCount += 1;
  }
});
badIter[Symbol.iterator] = function() {
  return {
    next: function() {
      return { done: false };
    },
    throw: function() {
      return spyValue;
    }
  };
};
var delegationComplete = false;
function* g() {
  yield * badIter;
  delegationComplete = true;
}
var iter = g();

iter.next();
assert.sameValue(callCount, 0, 'access count (first iteration)');
assert.sameValue(
  delegationComplete, false, 'delegation ongoing (first iteration)'
);

iter.throw();
assert.sameValue(callCount, 0, 'access count (second iteration)');
assert.sameValue(
  delegationComplete, false, 'delegation ongoing (second iteration)'
);

spyValue.done = true;
iter.throw();
assert.sameValue(callCount, 1, 'access count (final iteration)');
assert.sameValue(delegationComplete, true, 'delegation complete');
