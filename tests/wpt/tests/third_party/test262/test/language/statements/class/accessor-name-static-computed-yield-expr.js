// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object-initializer-runtime-semantics-evaluation
es6id: 12.2.6.8
description: >
  The `yield` keyword behaves as a YieldExpression within a generator function
info: |
  [...]
  21. For each ClassElement m in order from methods
      a. If IsStatic of m is false, then
         [...]
      b. Else,
         a. Let status be the result of performing PropertyDefinitionEvaluation
            for m with arguments F and false.

  ComputedPropertyName : [ AssignmentExpression ]

  1. Let exprValue be the result of evaluating AssignmentExpression.
  2. Let propName be ? GetValue(exprValue).
  3. Return ? ToPropertyKey(propName).
features: [generators]
---*/

var yieldSet, C, iter;
function* g() {
  class C_ {
    static get [yield]() { return 'get yield'; }
    static set [yield](param) { yieldSet = param; }
  }

  C = C_;
}

iter = g();

iter.next();
iter.next('first');
iter.next('second');

assert.sameValue(C.first, 'get yield');

C.second = 'set yield';

assert.sameValue(yieldSet, 'set yield');
