// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
es6id: 14.4.14
description: >
  Iteration protocol is not initiated for non-delegating YieldExpression
info: |
  YieldExpression:yieldAssignmentExpression

  1. Let exprRef be the result of evaluating AssignmentExpression.
  2. Let value be ? GetValue(exprRef).
  3. Return ? GeneratorYield(CreateIterResultObject(value, false)).
features: [generators, Symbol.iterator]
---*/

var callCount = 0;
var iterSpy = Object.defineProperty({}, Symbol.iterator, {
  get: function() {
    callCount += 1;
  }
});
function* g() {
  yield iterSpy;
}
var iter = g();
var result;

result = iter.next();

assert.sameValue(result.value, iterSpy);
assert.sameValue(result.done, false);
assert.sameValue(callCount, 0);

result = iter.next();

assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);
assert.sameValue(callCount, 0);
