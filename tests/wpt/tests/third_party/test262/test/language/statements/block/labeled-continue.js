// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-static-semantics-containsundefinedcontinuetarget
description: Clears label set in check for undefined `continue` target
info: |
  With arguments iterationSet and labelSet.

  Statement : BlockStatement

  1. Return ContainsUndefinedContinueTarget of |BlockStatement| with arguments
     _iterationSet_ and « ».
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

label: {
  for ( ;; ) {
    continue label;
  }
}
