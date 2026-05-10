// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-class-definitions-static-semantics-early-errors
description: BindingIdentifier may not be `await` within class static blocks
info: |
  BindingIdentifier : Identifier

  [...]
  - It is a Syntax Error if the code matched by this production is nested,
    directly or indirectly (but not crossing function or static initialization
    block boundaries), within a ClassStaticBlock and the StringValue of
    Identifier is "await".
negative:
  phase: parse
  type: SyntaxError
features: [class-static-block]
---*/

$DONOTEVALUATE();

class C {
  static {
    var await;
  }
}
