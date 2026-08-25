// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
es6id: 14.4.14
description: Abrupt completion returned when accessing the @@iterator property
info: |
  YieldExpression : yield * AssignmentExpression

  1. Let exprRef be the result of evaluating AssignmentExpression.
  2. Let value be ? GetValue(exprRef).
  3. Let iterator be ? GetIterator(value).

  7.4.1 GetIterator

  1. If method was not passed, then
     a. Let method be ? GetMethod(obj, @@iterator).
features: [generators, Symbol.iterator]
---*/

var thrown = new Test262Error();
var poisonedIter = Object.defineProperty({}, Symbol.iterator, {
  get: function() {
    throw thrown;
  }
});
function* g() {
  try {
    yield * poisonedIter;
  } catch (err) {
    caught = err;
  }
}
var iter = g();
var result, caught;

assert.sameValue(caught, undefined, 'property is not accessed eagerly');

result = iter.next();

assert.sameValue(result.value, undefined, 'iteration value');
assert.sameValue(result.done, true, 'iteration status');
assert.sameValue(caught, thrown, 'error value');
