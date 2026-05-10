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
  2. ...

  9.1.1.3.4 GetThisBinding ( )
    1. Assert: envRec.[[ThisBindingStatus]] is not lexical.
    2. If envRec.[[ThisBindingStatus]] is uninitialized, throw a ReferenceError exception.
    3. ...

features: [class, class-fields-private]
---*/

class C extends class {} {
  #field;

  constructor() {
    var init = () => super();

    var object = {
      get a() {
        init();
      }
    };

    // Accessing |this| should throw a ReferenceError before there's an attempt
    // to invoke the getter.
    ({a: this.#field} = object);
  }
}

assert.throws(ReferenceError, function() {
  new C();
});
