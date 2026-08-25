// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-delete-operator
description: Delete Operator throws an error if the base reference is not object-coercible (undefined).
info: |
  # 12.5.3.2 Runtime Semantics: Evaluation
  UnaryExpression : delete UnaryExpression

  [...]
  5. If IsPropertyReference(ref) is true, then
     a. If IsSuperReference(ref) is true, throw a ReferenceError exception.
     b. Let baseObj be ? ToObject(ref.[[Base]]).
---*/

var base = undefined;

assert.throws(TypeError, function() {
  delete base[0];
});
