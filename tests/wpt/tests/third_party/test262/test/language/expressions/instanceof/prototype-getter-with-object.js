// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 12.9.3
description: >
  "prototype" property is retrieved when left-hand side expression in `instanceof` is object.
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
    4. Let P be Get(C, "prototype").
    5. ReturnIfAbrupt(P).
    6. If Type(P) is not Object, throw a TypeError exception.
    7. Repeat
      a. Let O be O.[[GetPrototypeOf]]().
      b. ReturnIfAbrupt(O).
      c. If O is null, return false.
      d. If SameValue(P, O) is true, return true.
    ...
---*/

var getterCalled = false;

// The "prototype" property for constructor functions is a non-configurable data-property,
// therefore we need to use a non-constructor function to install the getter.
Object.defineProperty(Function.prototype, "prototype", {
  get: function() {
    assert.sameValue(getterCalled, false, "'prototype' getter called once");
    getterCalled = true;
    return Array.prototype;
  }
});

var result = [] instanceof Function.prototype;

assert(result, "[] should be instance of Function.prototype");
assert(getterCalled, "'prototype' getter called");
