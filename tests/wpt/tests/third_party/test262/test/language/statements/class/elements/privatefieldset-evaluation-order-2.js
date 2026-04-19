// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Evaluation order when resolving private fields.
esid: sec-runtime-semantics-keyeddestructuringassignmentevaluation
info: |
  13.15.5.6 Runtime Semantics: KeyedDestructuringAssignmentEvaluation
    1. If DestructuringAssignmentTarget is neither an ObjectLiteral nor an ArrayLiteral, then
      a. Let lref be the result of evaluating DestructuringAssignmentTarget.
      b. ReturnIfAbrupt(lref).
  2. Let v be ? GetV(value, propertyName).
  3. ...

features: [class, class-fields-private]
---*/

class C {
  #field;

  m() {
    var object = {
      get a() {
        throw new Test262Error();
      }
    };

    // The getter is executed before the check if the private field is present.
    ({a: this.#field} = object);
  }
}

assert.throws(Test262Error, function() {
  C.prototype.m.call({});
});
