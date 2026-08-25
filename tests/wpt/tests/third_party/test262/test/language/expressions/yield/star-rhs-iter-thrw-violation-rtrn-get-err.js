// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
es6id: 14.4.14
description: >
  Abrupt completion returned when accessing iterator `return` property after
  protocol violation
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
            [...]
        iii. Else,
             1. NOTE: If iterator does not have a throw method, this throw is
                going to terminate the yield* loop. But first we need to give
                iterator a chance to clean up.
             2. Perform ? IteratorClose(iterator, Completion{[[Type]]: normal,
                [[Value]]: empty, [[Target]]: empty}).

  7.4.6 IteratorClose

  1. Assert: Type(iterator) is Object.
  2. Assert: completion is a Completion Record.
  3. Let return be ? GetMethod(iterator, "return").
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
var caught;

iter.next();
iter.throw();

assert.sameValue(callCount, 1);
assert.sameValue(caught, thrown);
