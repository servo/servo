// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-break-statement
description: IterationStatement search does not traverse static initialization block boundaries (no label specified)
info: |
  4.2.1 Static Semantics: Early Errors
  BreakStatement : break ;

  - It is a Syntax Error if this BreakStatement is not nested, directly or
    indirectly (but not crossing function or static initialization block
    boundaries), within an IterationStatement or a SwitchStatement.
negative:
  phase: parse
  type: SyntaxError
features: [class-static-block]
---*/

$DONOTEVALUATE();

label: while(false) {
  class C {
    static {
      break;
    }
  }
}
