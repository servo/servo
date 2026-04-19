// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
es6id: 14.4.14
description: >
  TypeError thrown when iterator `throw` method returns a non-object value
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
features: [generators, Symbol.iterator]
---*/

var badIter = {};
badIter[Symbol.iterator] = function() {
  return {
    next: function() {
      return { done: false };
    },
    throw: function() {
      return 23;
    }
  };
};
function* g() {
  try {
    yield * badIter;
  } catch (err) {
    caught = err;
  }
}
var iter = g();
var result, caught;

iter.next();
result = iter.throw();

assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);
assert.sameValue(typeof caught, 'object');
assert.sameValue(caught.constructor, TypeError);
