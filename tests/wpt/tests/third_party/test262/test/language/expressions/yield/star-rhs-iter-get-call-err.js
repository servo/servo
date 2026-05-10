// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
es6id: 14.4.14
description: Abrupt completion returned when invoking the @@iterator method
info: |
  YieldExpression : yield * AssignmentExpression

  1. Let exprRef be the result of evaluating AssignmentExpression.
  2. Let value be ? GetValue(exprRef).
  3. Let iterator be ? GetIterator(value).

  7.4.1 GetIterator

  1. If method was not passed, then
     a. Let method be ? GetMethod(obj, @@iterator).
  2. Let iterator be ? Call(method, obj).
features: [generators, Symbol.iterator]
---*/

var thrown = new Test262Error();
var badIter = {};
badIter[Symbol.iterator] = function() {
  throw thrown;
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

assert.sameValue(caught, undefined, 'method is not invoked eagerly');

result = iter.next();

assert.sameValue(result.value, undefined, 'iteration value');
assert.sameValue(result.done, true, 'iteration status');
assert.sameValue(caught, thrown, 'error value');
