// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-tagged-templates-runtime-semantics-evaluation
es6id: 12.3.7.1
description: Value when invoked via tagged template
info: |
   MemberExpression : MemberExpression TemplateLiteral

   [...]
   4. Return ? EvaluateCall(tagRef, TemplateLiteral, tailCall).
features: [new.target, template]
---*/

var newTarget = null;

function f() {
  newTarget = new.target;
}

f``;

assert.sameValue(newTarget, undefined);
