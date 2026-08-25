// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-variable-statement-runtime-semantics-evaluation
es6id: 13.3.2.4
description: Binding is resolve prior to evaluation of Initializer
info: |
   VariableDeclaration : BindingIdentifier Initializer

   1. Let bindingId be StringValue of BindingIdentifier.
   2. Let lhs be ? ResolveBinding(bindingId).
   3. Let rhs be the result of evaluating Initializer.
   [...]
flags: [noStrict]
---*/

var obj = { test262id: 1 };

with (obj) {
  var test262id = delete obj.test262id;
}

assert.sameValue(obj.test262id, true);
assert.sameValue(test262id, undefined);
