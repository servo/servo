// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object-initializer-runtime-semantics-evaluation
es6id: 12.2.6.8
description: >
  AssignmentExpression may contain `in` keyword regardless of outer context
info: |
  12.2.6.7 Runtime Semantics: Evaluation

  [...]

  ComputedPropertyName : [ AssignmentExpression ]

  1. Let exprValue be the result of evaluating AssignmentExpression.
  2. Let propName be ? GetValue(exprValue).
  3. Return ? ToPropertyKey(propName).
---*/

var empty = Object.create(null);
var obj, value;

for (obj = { get ['x' in empty]() { return 'via get'; } }; ; ) {
  value = obj.false;
  break;
}

assert.sameValue(value, 'via get');

for (obj = { set ['x' in empty](param) { value = param; } }; ; ) {
  obj.false = 'via set';
  break;
}

assert.sameValue(value, 'via set');
