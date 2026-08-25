// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-conditional-operator-runtime-semantics-evaluation
description: >
  ToBoolean returns `false` for [[IsHTMLDDA]] object; trueRef is not evaluated.
info: |
  ConditionalExpression : ShortCircuitExpression ? AssignmentExpression : AssignmentExpression

  1. Let lref be the result of evaluating ShortCircuitExpression.
  2. Let lval be ! ToBoolean(? GetValue(lref)).
  3. If lval is true, then
    [...]
  4. Else,
    a. Let falseRef be the result of evaluating the second AssignmentExpression.
    b. Return ? GetValue(falseRef).

  The [[IsHTMLDDA]] Internal Slot / Changes to ToBoolean

  1. If argument has an [[IsHTMLDDA]] internal slot, return false.
  2. Return true.
features: [IsHTMLDDA]
---*/

assert.sameValue($262.IsHTMLDDA ? unresolved : 2, 2);
