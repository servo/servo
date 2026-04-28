// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
description: Value received from invocation of generator's `throw` method
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
               [...]
            6. Let received be GeneratorYield(innerResult).
features: [generators, Symbol.iterator]
---*/

var quickIter = {};
var iter, exprValue, throwReceived;
quickIter[Symbol.iterator] = function() {
  return {
    next: function() {
      return { done: false };
    },
    throw: function(x) {
      throwReceived = x;
      return { done: true, value: 3333 };
    }
  };
};
function* g() {
  exprValue = yield * quickIter;
}

iter = g();

iter.next();
iter.throw(2222);
assert.sameValue(throwReceived, 2222);
assert.sameValue(exprValue, 3333);
