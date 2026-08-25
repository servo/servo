// Copyright (C) 2020 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-new-operator
description: >
  The IsConstructor(ctor) happens after evaluating the arguments, use the correct ctor.
  Function wrap-up to use the same function level binding ref
info: |
  NewExpression : new NewExpression
    1. Return ? EvaluateNew(NewExpression, empty).
  MemberExpression : new MemberExpression Arguments
    1. Return ? EvaluateNew(MemberExpression, Arguments).

  Runtime Semantics: EvaluateNew

  3. Let ref be the result of evaluating constructExpr.
  4. Let constructor be ? GetValue(ref).
  5. If arguments is empty, let argList be a new empty List.
  6. Else,
    a. Let argList be ? ArgumentListEvaluation of arguments.
  7. If IsConstructor(constructor) is false, throw a TypeError exception.
  8. Return ? Construct(constructor, argList). 
---*/

var ref;
var argz;

assert.throws(TypeError, function() {
  var x = 42;
  ref = x;
  new x(x = function() {}, argz = 39);
});

assert.sameValue(ref, 42);
assert.sameValue(argz, 39, 'arguments evaluated before checking valid ctor');
