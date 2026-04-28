// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-new-operator-runtime-semantics-evaluation
es6id: 12.3.3.1
description: Value when invoked via NewExpression
info: |
  NewExpression:newNewExpression

  1. Return ? EvaluateNew(NewExpression, empty).

  MemberExpression:newMemberExpressionArguments

  2. Return ? EvaluateNew(MemberExpression, Arguments).

  12.3.3.1.1 Runtime Semantics: EvaluateNew

  [...]
  8. Return ? Construct(constructor, argList).

  7.3.13 Construct (F [ , argumentsList [ , newTarget ]])

  1. If newTarget was not passed, let newTarget be F.
features: [new.target]
---*/

var newTarget = null;
function f() {
  newTarget = new.target;
}

new f;

assert.sameValue(newTarget, f, 'Invoked without Arguments');

newTarget = null;

new f();

assert.sameValue(newTarget, f, 'Invoked with Arguments');
