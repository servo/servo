// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-property-accessors-runtime-semantics-evaluation
es6id: 12.3.2.1
description: Value when invoked via MemberExpression
info: |
  MemberExpression:MemberExpression.IdentifierName

  [...]
  6. Return a value of type Reference whose base value component is bv, whose
     referenced name component is propertyNameString, and whose strict
     reference flag is strict.

  13.5.1 Runtime Semantics: Evaluation

  ExpressionStatement : Expression ;

  1. Let exprRef be the result of evaluating Expression.
  2. Return ? GetValue(exprRef).
features: [new.target]
---*/

var newTarget = null;

var obj = {
  get m() {
    newTarget = new.target;
  }
};

obj.m;

assert.sameValue(newTarget, undefined);
