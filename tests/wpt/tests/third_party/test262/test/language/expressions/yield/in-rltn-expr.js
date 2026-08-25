// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions
es6id: 14.4
description: >
  YieldExpression contextually recognizes the `in` keyword as part of a
  RelationalExpression
info: |
  Syntax

  yield [no LineTerminator here] AssignmentExpression[?In, +Yield]
features: [generators]
---*/

var obj = Object.create(null);
var iter, iterResult, value;
function* g() {
  value = yield 'hit' in obj;
  value = yield 'miss' in obj;
}
obj.hit = true;

iter = g();

iterResult = iter.next('first');

assert.sameValue(iterResult.done, false, '`done` property (first iteration)');
assert.sameValue(iterResult.value, true, '`value` property (first iteration)');
assert.sameValue(
  value, undefined, 'generator paused prior to evaluating AssignmentExpression'
);

iterResult = iter.next('second');

assert.sameValue(iterResult.done, false, '`done` property (second iteration)');
assert.sameValue(
  iterResult.value, false, '`value` property (second iteration)'
);
assert.sameValue(value, 'second', 'value of first AssignmentExpression');

iterResult = iter.next('third');

assert.sameValue(iterResult.done, true, '`done` property (third iteration)');
assert.sameValue(
  iterResult.value, undefined, '`value` property (third iteration)'
);
assert.sameValue(value, 'third', 'value of second AssignmentExpression');
