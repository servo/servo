// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-assignment-operators-runtime-semantics-evaluation
description: >
  ToBoolean returns `false` for [[IsHTMLDDA]] object; rval is evaluated.
info: |
  AssignmentExpression : LeftHandSideExpression ||= AssignmentExpression

  1. Let lref be the result of evaluating LeftHandSideExpression.
  2. Let lval be ? GetValue(lref).
  3. Let lbool be ! ToBoolean(lval).
  [...]
  7. Perform ? PutValue(lref, rval).
  8. Return rval.

  The [[IsHTMLDDA]] Internal Slot / Changes to ToBoolean

  1. If argument has an [[IsHTMLDDA]] internal slot, return false.
  2. Return true.
features: [IsHTMLDDA, logical-assignment-operators]
---*/

var value = $262.IsHTMLDDA;
assert.sameValue(value ||= 2, 2);
assert.sameValue(value, 2);
