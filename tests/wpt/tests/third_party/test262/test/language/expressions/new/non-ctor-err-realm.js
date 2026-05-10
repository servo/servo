// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-new-operator
es6id: 12.3.3
description: Realm of error object produced when operand is not a constructor
info: |
  NewExpression : new NewExpression

  1. Return ? EvaluateNew(NewExpression, empty).

  MemberExpression : new MemberExpression Arguments

  1. Return ? EvaluateNew(MemberExpression, Arguments).

  12.3.3.1.1 Runtime Semantics: EvaluateNew

  7. If IsConstructor(constructor) is false, throw a TypeError exception.
features: [cross-realm]
---*/

var otherParseInt = $262.createRealm().global.parseInt;

assert.sameValue(typeof otherParseInt, 'function');

assert.throws(TypeError, function() {
  new otherParseInt(0);
}, 'production including Arguments');

assert.throws(TypeError, function() {
  new otherParseInt;
}, 'production eliding Arguments');
