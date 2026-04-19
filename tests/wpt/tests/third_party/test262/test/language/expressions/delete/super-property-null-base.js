// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-delete-operator-runtime-semantics-evaluation
description: The restriction on the base of a super property must not be enforced before the delete expression is evaluated.
info: |
  # 13.3.7.3 MakeSuperPropertyReference ( actualThis, propertyKey, strict )

  1. Let env be GetThisEnvironment().
  2. Assert: env.HasSuperBinding() is true.
  3. Let baseValue be ? env.GetSuperBase().
  4. Let bv be ? RequireObjectCoercible(baseValue).

  # 13.5.1.2 Runtime Semantics: Evaluation
  UnaryExpression : delete UnaryExpression

  1. Let ref be the result of evaluating UnaryExpression.
  2. ReturnIfAbrupt(ref).
  [...]
  5. If IsPropertyReference(ref) is true, then
    a. Assert: ! IsPrivateReference(ref) is false.
    b. If IsSuperReference(ref) is true, throw a ReferenceError exception.
features: [class]
---*/

class C {
  static m() {
    delete super.x;
  }
}

Object.setPrototypeOf(C, null);

assert.throws(ReferenceError, () => {
  C.m();
});
