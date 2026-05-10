// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 12.9.3
description: >
  Does not throw a TypeError if left-hand side expression and `prototype` property are both primitive values.
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

// Check with primitive "prototype" property on non-constructor function.
Function.prototype.prototype = true;

var result = 0 instanceof Function.prototype;

assert.sameValue(result, false);
