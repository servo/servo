// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-delete-operator-runtime-semantics-evaluation
description: >
  Element expression in delete super not evaluated when this is uninitialized.
info: |
  13.5.1.2 Runtime Semantics: Evaluation

    UnaryExpression : delete UnaryExpression

    1. Let ref be ? Evaluation of UnaryExpression.
    ...

  13.3.7.1 Runtime Semantics: Evaluation

    SuperProperty : super [ Expression ]

    ...
    2. Let actualThis be ? env.GetThisBinding().
    3. Let propertyNameReference be ? Evaluation of Expression.
    ...

  9.1.1.3.4 GetThisBinding ( )
    ...
    2. If envRec.[[ThisBindingStatus]] is uninitialized, throw a ReferenceError exception.
    ...
---*/

class Base {
  constructor() {
    throw new Test262Error("base constructor called");
  }
}

class Derived extends Base {
  constructor() {
    delete super[(super(), 0)];
  }
}

assert.throws(ReferenceError, () => new Derived);
