// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 12.9.3
description: >
  "prototype" property is not retrieved when left-hand side expression in `instanceof` is primitive.
info: |
  12.9.3 Runtime Semantics: Evaluation
  RelationalExpression : RelationalExpression instanceof ShiftExpression
    ...
    7. Return InstanceofOperator(lval, rval).

    12.9.4 Runtime Semantics: InstanceofOperator(O, C)
    ...
    6. Return OrdinaryHasInstance(C, O).

    7.3.19 OrdinaryHasInstance
    ...
    3. If Type(O) is not Object, return false.
    ...
---*/

// The "prototype" property for constructor functions is a non-configurable data-property,
// therefore we need to use a non-constructor function to install the getter.
Object.defineProperty(Function.prototype, "prototype", {
  get: function() {
    throw new Test262Error("getter for 'prototype' called");
  }
});

var result = 0 instanceof Function.prototype;

assert.sameValue(result, false);
