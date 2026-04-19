// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
es6id: 14.4.14
description: Abrupt completion returned when accessing iterator `return` method
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
        [...]
     c. Else,
        i. Assert: received.[[Type]] is return.
        ii. Let return be ? GetMethod(iterator, "return").
features: [generators, Symbol.iterator]
---*/

var thrown = new Test262Error();
var badIter = {};
var callCount = 0;
var poisonedReturn = {
  next: function() {
    return { done: false };
  }
};
Object.defineProperty(poisonedReturn, 'throw', {
  get: function() {
    callCount += 1;
  }
});
Object.defineProperty(poisonedReturn, 'return', {
  get: function() {
    throw thrown;
  }
});
badIter[Symbol.iterator] = function() {
  return poisonedReturn;
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

assert.sameValue(caught, undefined, '`return` property not accessed eagerly');

result = iter.return();

assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);
assert.sameValue(caught, thrown);
assert.sameValue(callCount, 0, 'iterator `throw` property is not accessed');
