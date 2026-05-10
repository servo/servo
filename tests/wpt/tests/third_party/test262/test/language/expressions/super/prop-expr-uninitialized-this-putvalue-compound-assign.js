// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-super-keyword-runtime-semantics-evaluation
description: >
  Expression not evaluated when this binding is uninitialized in PutValue context with compound assignment.
info: |
  13.3.7.1 Runtime Semantics: Evaluation

    SuperProperty : super [ Expression ]

    ...
    2. Let actualThis be ? env.GetThisBinding().
    3. Let propertyNameReference be ? Evaluation of Expression.
    ...
---*/

class Base {
  constructor() {
    throw new Test262Error("base constructor");
  }
}

class Derived extends Base {
  constructor() {
    super[super()] += 0;
  }
}

assert.throws(ReferenceError, () => new Derived);
