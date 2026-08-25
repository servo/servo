// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object-initializer-runtime-semantics-evaluation
es6id: 12.2.6.8
description: >
  The `yield` keyword behaves as a YieldExpression within a generator function
info: |
  12.2.6.7 Runtime Semantics: Evaluation

  [...]

  ComputedPropertyName : [ AssignmentExpression ]

  1. Let exprValue be the result of evaluating AssignmentExpression.
  2. Let propName be ? GetValue(exprValue).
  3. Return ? ToPropertyKey(propName).
features: [generators]
---*/

var yieldSet, obj, iter;
function* g() {
  obj = {
    get [yield]() { return 'get yield'; },
    set [yield](param) { yieldSet = param; }
  };
}

iter = g();

iter.next();
iter.next('first');
iter.next('second');

assert.sameValue(obj.first, 'get yield');

obj.second = 'set yield';

assert.sameValue(yieldSet, 'set yield');
