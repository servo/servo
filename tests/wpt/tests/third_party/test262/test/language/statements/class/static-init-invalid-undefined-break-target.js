// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-class-definitions-static-semantics-early-errors
description: Block cannot reference an undefined `break` target
info: |
  ClassStaticBlockBody : ClassStaticBlockStatementList

  - It is a Syntax Error if ContainsUndefinedBreakTarget of
    ClassStaticBlockStatementList with argument « » is true.
negative:
  phase: parse
  type: SyntaxError
features: [class-static-block]
---*/

$DONOTEVALUATE();

class C {
  static {
    x: while (false) {
      break y;
    }
  }
}
