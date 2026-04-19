// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-binary-logical-operators-runtime-semantics-evaluation
description: >
  ToBoolean returns `false` for [[IsHTMLDDA]] object; rval is evaluated.
info: |
  LogicalORExpression : LogicalORExpression || LogicalANDExpression

  1. Let lref be the result of evaluating LogicalORExpression.
  2. Let lval be ? GetValue(lref).
  3. Let lbool be ! ToBoolean(lval).
  4. If lbool is true, return lval.
  5. Let rref be the result of evaluating LogicalANDExpression.
  6. Return ? GetValue(rref).

  The [[IsHTMLDDA]] Internal Slot / Changes to ToBoolean

  1. If argument has an [[IsHTMLDDA]] internal slot, return false.
  2. Return true.
features: [IsHTMLDDA]
---*/

assert.sameValue($262.IsHTMLDDA || 2, 2);
