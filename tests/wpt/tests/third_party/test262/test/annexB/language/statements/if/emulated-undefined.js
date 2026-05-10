// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-if-statement-runtime-semantics-evaluation
description: >
  ToBoolean returns `false` for [[IsHTMLDDA]] object; first Statement is not evaluated.
info: |
  IfStatement : if ( Expression ) Statement else Statement

  1. Let exprRef be the result of evaluating Expression.
  2. Let exprValue be ! ToBoolean(? GetValue(exprRef)).
  3. If exprValue is true, then
    [...]
  4. Else,
    a. Let stmtCompletion be the result of evaluating the second Statement.

  The [[IsHTMLDDA]] Internal Slot / Changes to ToBoolean

  1. If argument has an [[IsHTMLDDA]] internal slot, return false.
  2. Return true.
features: [IsHTMLDDA]
---*/

var result = false;
if ($262.IsHTMLDDA) {
  throw new Test262Error("unreachable");
} else {
  result = true;
}

assert(result);
