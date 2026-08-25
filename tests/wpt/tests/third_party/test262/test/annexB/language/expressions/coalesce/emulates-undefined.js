// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-binary-bitwise-operators-runtime-semantics-evaluation
description: >
  ?? doesn't special-case [[IsHTMLDDA]] object; rval is not evaluated.
info: |
  CoalesceExpression : CoalesceExpressionHead ?? BitwiseORExpression

  1. Let lref be the result of evaluating CoalesceExpressionHead.
  2. Let lval be ? GetValue(lref).
  3. If lval is undefined or null, then
    [...]
  4. Otherwise, return lval.
features: [IsHTMLDDA, coalesce-expression]
---*/

var IsHTMLDDA = $262.IsHTMLDDA;

assert.sameValue(IsHTMLDDA ?? unresolved, IsHTMLDDA);
